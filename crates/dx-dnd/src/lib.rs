//! Multi-target drag-and-drop primitives for Dioxus 0.7.
//!
//! Built on HTML5 native drag-and-drop (`draggable="true"`,
//! `ondragstart`, `ondragover`, `ondrop`, `ondragend`). The browser
//! owns the drag preview and the end-of-drag, so there's no "lost in
//! space" failure mode — `ondragend` fires regardless of where the
//! pointer was released.
//!
//! Two ways to use:
//!
//! - **Low-level**: call [`use_drag_drop`] to get a [`DragDrop`]
//!   handle plus a [`DropTargetCtx`] in context, and wire the
//!   per-event factories ([`DragDrop::on_drag_start`],
//!   [`DragDrop::on_drag_end`], plus `set_drop_target` / `commit_drop`
//!   from the context) onto your own elements.
//! - **High-level**: wrap your draggable area in [`DragDropArea`],
//!   your lists in [`DropList`], and your items in [`Draggable`].
//!
//! Works on desktop, iPad, and Android — HTML5 drag-and-drop is
//! supported natively on iOS Safari 15+ and modern Android browsers,
//! so no polyfill is needed.
//!
//! See `examples/basic` for a working 3-column board.

use dioxus::prelude::*;

mod components;
mod core;
mod state;

pub use components::{
    DragDropArea, DragDropAreaProps, Draggable, DraggableProps, DropList, DropListProps,
};
pub use state::{
    DragDrop, DragDropConfig, DragDropEvent, DragDropState, DragStart, DropTargetCtx,
    use_drag_drop, use_drag_drop_ctx,
};

/// Default stylesheet (`.dnd-dz`, `.dnd-dz-hover`, `.dnd-dz-tail`).
pub const DEFAULT_STYLE: Asset = asset!("/assets/dx-dnd.css");
