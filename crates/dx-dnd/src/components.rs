use dioxus::prelude::*;

use crate::state::{
    DragDrop, DragDropConfig, DragDropEvent, DropTargetCtx, use_drag_drop, use_drag_drop_ctx,
};

/// Top-level wrapper. Initializes drag-drop state and provides it via
/// context to descendants. No drag handlers live here — HTML5 native
/// drag-and-drop handles dispatching; we just provide the shared state.
#[component]
pub fn DragDropArea<ItemId: Copy + Eq + 'static>(
    #[props(default)] config: DragDropConfig,
    #[props(default)] class: String,
    #[props(default)] style: String,
    on_drop: EventHandler<DragDropEvent<ItemId>>,
    children: Element,
) -> Element {
    let _ = use_drag_drop::<ItemId>(config, move |evt| on_drop.call(evt));

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            {children}
        }
    }
}

/// A drop target. Renders a container plus a "tail" drop zone that
/// catches drops past the last item (or inside an empty list).
#[component]
pub fn DropList(
    list_id: String,
    count: usize,
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            "data-dnd-list": "{list_id}",
            {children}
            TailZone { list_id: list_id.clone(), count }
        }
    }
}

/// The tail drop zone of a [`DropList`]. Lives in its own component so
/// drop-target hover state only re-renders this small node.
#[component]
fn TailZone(list_id: String, count: usize) -> Element {
    let ctx = use_context::<DropTargetCtx>();
    let is_hover = match &*ctx.drop_target.read() {
        Some((l, s)) => l == &list_id && *s == count,
        None => false,
    };
    let class = if is_hover {
        "dnd-dz dnd-dz-tail dnd-dz-hover"
    } else {
        "dnd-dz dnd-dz-tail"
    };
    let over_list = list_id.clone();
    let drop_list = list_id.clone();
    rsx! {
        div {
            class,
            "data-dnd-list": "{list_id}",
            "data-dnd-slot": "{count}",
            ondragover: move |e: DragEvent| {
                e.prevent_default();
                ctx.set_drop_target.call((over_list.clone(), count));
            },
            ondrop: move |e: DragEvent| {
                e.prevent_default();
                ctx.commit_drop.call((drop_list.clone(), count));
            },
        }
    }
}

/// A draggable item. Renders a drop-zone sliver above the item (which
/// lights up when this slot is the active drop target) followed by the
/// item itself with `draggable="true"`. The browser handles the drag
/// preview and gesture; we just track the source and react to drops.
///
/// The item div is itself a drop target: dropping on the top half
/// inserts at this slot, dropping on the bottom half inserts at
/// `slot + 1`. This means in-list reorder works by dropping anywhere
/// on the row (not just the 10px sliver between rows).
#[component]
pub fn Draggable<ItemId: Copy + Eq + 'static>(
    item_id: ItemId,
    list_id: String,
    slot: usize,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let dnd: DragDrop<ItemId> = use_drag_drop_ctx::<ItemId>();
    let ctx = use_context::<DropTargetCtx>();
    let is_dragging = dnd.is_dragging(item_id);
    let style = if is_dragging {
        "opacity: 0.4; transition: opacity 120ms ease;"
    } else {
        "opacity: 1; transition: opacity 120ms ease;"
    };

    let over_list = list_id.clone();
    let drop_list = list_id.clone();

    rsx! {
        SlotZone { list_id: list_id.clone(), slot }
        div {
            class: "{class}",
            style: "{style}",
            "data-dnd-item": "true",
            "data-dnd-list": "{list_id}",
            "data-dnd-slot": "{slot}",
            ondragend: dnd.on_drag_end(),
            ondragover: move |e: DragEvent| {
                e.prevent_default();
                let target_slot = midline_slot(&e, slot);
                ctx.set_drop_target.call((over_list.clone(), target_slot));
            },
            ondrop: move |e: DragEvent| {
                e.prevent_default();
                let target_slot = midline_slot(&e, slot);
                ctx.commit_drop.call((drop_list.clone(), target_slot));
            },
            // Only the handle is `draggable=true`, so clicks elsewhere
            // on the card (text, buttons, inputs) don't initiate drag.
            // `on_drag_start` calls `setDragImage` on the nearest
            // `[data-dnd-item]` ancestor (this outer div) so the
            // whole card is what visually drags.
            span {
                class: "dnd-handle",
                draggable: "true",
                ondragstart: dnd.on_drag_start(item_id, list_id.clone(), slot),
                "⠿"
            }
            {children}
        }
    }
}

/// Map a dragover/drop event on a `[data-dnd-item]` ancestor to a slot
/// index: above the item's vertical midline returns `slot`, below
/// returns `slot + 1`. Falls back to `slot` on non-web targets or when
/// the bounding rect can't be read.
#[allow(unused_variables)]
fn midline_slot(e: &DragEvent, slot: usize) -> usize {
    #[cfg(feature = "web")]
    {
        use dioxus::web::WebEventExt;
        use wasm_bindgen::JsCast;
        if let Some(web_evt) = e.data().try_as_web_event() {
            let client_y = web_evt.client_y() as f64;
            if let Some(target) = web_evt.target() {
                if let Ok(start_el) = target.dyn_into::<web_sys::Element>() {
                    if let Ok(Some(item_el)) = start_el.closest("[data-dnd-item]") {
                        if let Ok(html_el) = item_el.dyn_into::<web_sys::HtmlElement>() {
                            let rect = html_el.get_bounding_client_rect();
                            let midline = rect.top() + rect.height() / 2.0;
                            return if client_y < midline { slot } else { slot + 1 };
                        }
                    }
                }
            }
        }
    }
    slot
}

/// The drop-zone sliver rendered above each [`Draggable`]. Like
/// `TailZone`, it lives in its own component so drop-target hover
/// state only re-renders this small node.
#[component]
fn SlotZone(list_id: String, slot: usize) -> Element {
    let ctx = use_context::<DropTargetCtx>();
    let is_hover = match &*ctx.drop_target.read() {
        Some((l, s)) => l == &list_id && *s == slot,
        None => false,
    };
    let class = if is_hover {
        "dnd-dz dnd-dz-hover"
    } else {
        "dnd-dz"
    };
    let over_list = list_id.clone();
    let drop_list = list_id.clone();
    rsx! {
        div {
            class,
            "data-dnd-list": "{list_id}",
            "data-dnd-slot": "{slot}",
            ondragover: move |e: DragEvent| {
                e.prevent_default();
                ctx.set_drop_target.call((over_list.clone(), slot));
            },
            ondrop: move |e: DragEvent| {
                e.prevent_default();
                ctx.commit_drop.call((drop_list.clone(), slot));
            },
        }
    }
}
