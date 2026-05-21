//! Pure (no-DOM, no-signal) helpers backing the drag-drop runtime.
//!
//! These are the parts of the logic that don't need a browser to run,
//! which makes them unit-testable from plain `cargo test`. The
//! signal-driven runtime in `state.rs` calls into these.

use crate::state::{DragDropEvent, DragStart};

/// Build a [`DragDropEvent`] from the recorded drag source + the slot
/// that received the drop. Returns `None` if no source was recorded
/// (e.g. drop fired without a prior dragstart, which shouldn't happen
/// but is cheap to guard against).
pub fn derive_drop_event<ItemId: Copy + Eq>(
    start: Option<&DragStart<ItemId>>,
    target: Option<(String, usize)>,
) -> Option<DragDropEvent<ItemId>> {
    let start = start?;
    let (to_list, to_slot) = target?;
    Some(DragDropEvent {
        item_id: start.item,
        from_list: start.list.clone(),
        from_slot: start.slot,
        to_list,
        to_slot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start(item: i64, list: &str, slot: usize) -> DragStart<i64> {
        DragStart {
            item,
            list: list.into(),
            slot,
        }
    }

    #[test]
    fn happy_path_across_lists() {
        let s = start(42, "todo", 1);
        let evt = derive_drop_event(Some(&s), Some(("done".into(), 0))).expect("drop");
        assert_eq!(evt.item_id, 42);
        assert_eq!(evt.from_list, "todo");
        assert_eq!(evt.from_slot, 1);
        assert_eq!(evt.to_list, "done");
        assert_eq!(evt.to_slot, 0);
    }

    #[test]
    fn within_same_list_preserves_from_and_to() {
        // Reorder within the same list — consumer is responsible for
        // any same-list slot adjustment, the event reports the raw
        // user intent.
        let s = start(7, "todo", 2);
        let evt = derive_drop_event(Some(&s), Some(("todo".into(), 5))).expect("drop");
        assert_eq!(evt.from_list, "todo");
        assert_eq!(evt.from_slot, 2);
        assert_eq!(evt.to_list, "todo");
        assert_eq!(evt.to_slot, 5);
    }

    #[test]
    fn returns_none_without_recorded_start() {
        // Defensive: ondrop without a preceding ondragstart shouldn't
        // produce a bogus event.
        let evt = derive_drop_event::<i64>(None, Some(("done".into(), 0)));
        assert!(evt.is_none());
    }

    #[test]
    fn returns_none_without_target() {
        // Won't happen via the HTML5 ondrop path (the target is always
        // known when ondrop fires) — but the API leaves room for
        // pointerup-style callers to pass `None`, and we should
        // suppress the drop cleanly in that case.
        let s = start(1, "todo", 0);
        let evt = derive_drop_event(Some(&s), None);
        assert!(evt.is_none());
    }
}
