use dioxus::prelude::*;

use crate::core::derive_drop_event;

/// What the user grabbed: which item, from which list, and where in
/// that list it was sitting. Recorded on `ondragstart` and cleared on
/// `ondragend`.
#[derive(Clone, Debug)]
pub struct DragStart<ItemId> {
    pub item: ItemId,
    pub list: String,
    pub slot: usize,
}

/// Tunables for [`use_drag_drop`]. Reserved for future knobs; nothing
/// to configure today (the browser handles drag-preview / threshold /
/// hit-testing natively).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DragDropConfig {}

/// Fired on a successful drop. The consumer's `on_drop` callback
/// moves `from_list[from_slot]` to `to_list[to_slot]`.
#[derive(Clone, Debug)]
pub struct DragDropEvent<ItemId> {
    pub item_id: ItemId,
    pub from_list: String,
    pub from_slot: usize,
    pub to_list: String,
    pub to_slot: usize,
}

/// Shared signals backing an active drag. Created by [`use_drag_drop`]
/// and provided via context.
pub struct DragDropState<ItemId: 'static> {
    /// What the user is currently dragging, or `None` if no drag is
    /// active. Used by [`crate::Draggable`] to know which item is the
    /// drag source (so it can dim, etc.).
    pub dragged: Signal<Option<DragStart<ItemId>>>,
    /// Active drop slot: `Some((list_id, slot))` while the pointer is
    /// over a drop zone, `None` otherwise.
    pub drop_target: Signal<Option<(String, usize)>>,
}

/// Non-generic projection of the drop-zone interactions. Provided as
/// a separate context so non-generic descendants (e.g. the sliver
/// components, `<DropList>`) can wire `ondragover` / `ondrop` without
/// naming `ItemId`. Callbacks close over the typed source state and
/// fire the consumer's `on_drop` internally.
#[derive(Clone, Copy)]
pub struct DropTargetCtx {
    pub drop_target: Signal<Option<(String, usize)>>,
    /// Idempotent: sets the hover slot if it's not already set.
    pub set_drop_target: Callback<(String, usize)>,
    /// Reads the recorded drag source, clears it, and fires the
    /// consumer's `on_drop` with the (list, slot) being dropped onto.
    pub commit_drop: Callback<(String, usize)>,
}

impl<ItemId: 'static> Clone for DragDropState<ItemId> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<ItemId: 'static> Copy for DragDropState<ItemId> {}

/// Handle returned by [`use_drag_drop`]. `Copy`, so it can be captured
/// freely in event handlers.
pub struct DragDrop<ItemId: Copy + Eq + 'static> {
    pub state: DragDropState<ItemId>,
    pub on_drop: Callback<DragDropEvent<ItemId>>,
}

impl<ItemId: Copy + Eq + 'static> Clone for DragDrop<ItemId> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<ItemId: Copy + Eq + 'static> Copy for DragDrop<ItemId> {}

/// Initialize drag-drop state inside a component. Provides the state
/// via context so descendants ([`crate::DropList`] / [`crate::Draggable`])
/// can see it.
pub fn use_drag_drop<ItemId>(
    _config: DragDropConfig,
    on_drop: impl FnMut(DragDropEvent<ItemId>) + 'static,
) -> DragDrop<ItemId>
where
    ItemId: Copy + Eq + 'static,
{
    let state = use_context_provider(|| DragDropState::<ItemId> {
        dragged: Signal::new(None),
        drop_target: Signal::new(None),
    });
    let on_drop = use_callback(on_drop);
    let set_drop_target: Callback<(String, usize)> = use_callback(move |(list, slot)| {
        let mut drop_target = state.drop_target;
        let next = Some((list, slot));
        if *drop_target.read() != next {
            drop_target.set(next);
        }
    });
    let commit_drop: Callback<(String, usize)> = use_callback(move |(list, slot)| {
        let mut state = state;
        let start = state.dragged.read().clone();
        state.dragged.set(None);
        state.drop_target.set(None);
        if let Some(evt) = derive_drop_event(start.as_ref(), Some((list, slot))) {
            on_drop.call(evt);
        }
    });
    let _ = use_context_provider(|| DropTargetCtx {
        drop_target: state.drop_target,
        set_drop_target,
        commit_drop,
    });
    let handle = DragDrop { state, on_drop };
    let _ = use_context_provider(|| handle);
    handle
}

/// Fetch the [`DragDrop`] handle from context inside a child component.
pub fn use_drag_drop_ctx<ItemId>() -> DragDrop<ItemId>
where
    ItemId: Copy + Eq + 'static,
{
    use_context::<DragDrop<ItemId>>()
}

impl<ItemId: Copy + Eq + 'static> DragDrop<ItemId> {
    /// Whether this specific item is the one currently being dragged.
    /// Useful for dimming the drag source while the user drags it.
    pub fn is_dragging(&self, item: ItemId) -> bool {
        matches!(&*self.state.dragged.read(), Some(d) if d.item == item)
    }

    /// Whether `(list, slot)` is the active drop target.
    pub fn is_drop_target(&self, list: &str, slot: usize) -> bool {
        match &*self.state.drop_target.read() {
            Some((l, s)) => l == list && *s == slot,
            None => false,
        }
    }

    /// `ondragstart` handler — records what was grabbed. Hand this to
    /// the element that has `draggable: "true"` (whether that's the
    /// whole card or a small handle inside it).
    ///
    /// On web, walks up from the event target to the nearest
    /// `[data-dnd-item]` ancestor and calls `setDragImage` on it, so
    /// when the grab happens on a small handle the **entire card**
    /// drags as the preview. Falls back to the browser's default
    /// (drag the element with `draggable=true`) if there's no item
    /// ancestor.
    pub fn on_drag_start(
        self,
        item: ItemId,
        list: String,
        slot: usize,
    ) -> impl FnMut(DragEvent) + 'static {
        let mut state = self.state;
        move |_e: DragEvent| {
            state.dragged.set(Some(DragStart {
                item,
                list: list.clone(),
                slot,
            }));
            state.drop_target.set(None);

            #[cfg(feature = "web")]
            {
                use dioxus::web::WebEventExt;
                use wasm_bindgen::JsCast;
                if let Some(web_evt) = _e.data().try_as_web_event() {
                    if let Some(dt) = web_evt.data_transfer() {
                        // Firefox requires any setData to actually start a drag.
                        let _ = dt.set_data("text/plain", "dx-dnd");
                        if let Some(target) = web_evt.target() {
                            if let Ok(start_el) = target.dyn_into::<web_sys::Element>() {
                                if let Ok(Some(item_el)) = start_el.closest("[data-dnd-item]") {
                                    if let Ok(html_el) = item_el.dyn_into::<web_sys::HtmlElement>()
                                    {
                                        dt.set_drag_image(&html_el, 0, 0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// `ondragend` handler — fires on drag end regardless of where the
    /// pointer was released (over a drop zone, in empty space, off the
    /// page entirely). Always clears state, so the source item can
    /// never get stranded.
    pub fn on_drag_end(self) -> impl FnMut(DragEvent) + 'static {
        let mut state = self.state;
        move |_e: DragEvent| {
            state.dragged.set(None);
            state.drop_target.set(None);
        }
    }
}
