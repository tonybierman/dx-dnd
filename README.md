# Dioxus Drag and Drop

Multi-target (cross-container) drag-and-drop primitives for Dioxus 0.7.
Pointer-events based — works on mouse, touch, and pen with a single code
path.

## Usage

Low-level (the hook):

```rust
use dx_dnd::{use_drag_drop, DragDropConfig, DragDropEvent};

let dnd = use_drag_drop::<i64>(
    DragDropConfig::default(),
    move |evt: DragDropEvent<i64>| {
        // mutate your state
    },
);

// On a parent: spread pointermove / pointerup / pointercancel
// On each item: dnd.start_drag(item_id, list_id, slot) as onpointerdown
//               dnd.item_style(item_id) as inline style
// On each drop slot: read dnd.is_drop_target(list_id, slot) for hover
```

High-level (the components):

```rust
use dx_dnd::{DragDropArea, DropList, Draggable};

DragDropArea::<i64> {
    on_drop: move |evt| { /* mutate state */ },
    for (list_id, items) in lists {
        DropList {
            list_id: list_id.clone(),
            count: items.len(),
            for (idx, item) in items.iter().enumerate() {
                Draggable::<i64> {
                    item_id: item.id,
                    list_id: list_id.clone(),
                    slot: idx,
                    "{item.label}"
                }
            }
        }
    }
}
```

## Example

```
cd examples/basic
dx serve --platform web
```

Open the URL it prints in a browser. Iterate.

## Status

Early. The library powers `dx_standup`'s board after migration (separate
session). Not published.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
