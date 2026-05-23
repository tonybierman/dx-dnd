//! Standalone harness for `dx-dnd`. Run with `dx serve --platform web`.
//!
//! Three columns ("Todo" / "Doing" / "Done"), each holding a
//! `Vec<i64>` of item ids. Drop events mutate the state in place. A
//! header bar lets you add items to each column so you can poke at edge
//! cases (empty column, single item, long column, etc.).

use dioxus::prelude::*;

use dx_dnd::{DEFAULT_STYLE, DragDropArea, DragDropEvent, Draggable, DropList};

const APP_CSS: Asset = asset!("/assets/app.css");

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq, Eq)]
struct Item {
    id: i64,
    label: String,
}

type Lists = Vec<(String, Vec<Item>)>;

#[component]
fn App() -> Element {
    let mut next_id = use_signal(|| 100i64);
    let mut lists: Signal<Lists> = use_signal(|| {
        vec![
            (
                "todo".to_string(),
                vec![
                    Item {
                        id: 1,
                        label: "Write the test plan".into(),
                    },
                    Item {
                        id: 2,
                        label: "Set up the staging environment".into(),
                    },
                    Item {
                        id: 3,
                        label: "Draft the launch email".into(),
                    },
                ],
            ),
            (
                "doing".to_string(),
                vec![
                    Item {
                        id: 4,
                        label: "Polish the demo deck".into(),
                    },
                    Item {
                        id: 5,
                        label: "Review the PR".into(),
                    },
                ],
            ),
            (
                "done".to_string(),
                vec![Item {
                    id: 6,
                    label: "Kickoff meeting".into(),
                }],
            ),
        ]
    });

    let on_drop = move |evt: DragDropEvent<i64>| {
        // Snapshot what we're moving, mutate, then sort.
        let mut snap = lists.write();
        let from_idx = snap.iter().position(|(id, _)| id == &evt.from_list);
        let to_idx = snap.iter().position(|(id, _)| id == &evt.to_list);
        let (Some(from_idx), Some(to_idx)) = (from_idx, to_idx) else {
            return;
        };
        if evt.from_slot >= snap[from_idx].1.len() {
            return;
        }
        let item = snap[from_idx].1.remove(evt.from_slot);
        // If we removed from earlier in the same list, the to_slot shifts left by 1.
        let to_slot = if from_idx == to_idx && evt.from_slot < evt.to_slot {
            evt.to_slot - 1
        } else {
            evt.to_slot
        };
        let to_slot = to_slot.min(snap[to_idx].1.len());
        snap[to_idx].1.insert(to_slot, item);
        web_sys_log(&format!(
            "drop: item {} from {}[{}] to {}[{}]",
            evt.item_id, evt.from_list, evt.from_slot, evt.to_list, evt.to_slot
        ));
    };

    let add_to = move |list_id: String| {
        let id = *next_id.read();
        next_id.set(id + 1);
        let mut snap = lists.write();
        if let Some((_, items)) = snap.iter_mut().find(|(l, _)| l == &list_id) {
            items.push(Item {
                id,
                label: format!("New item {id}"),
            });
        }
    };

    let lists_snapshot = lists.read().clone();

    rsx! {
        document::Link { rel: "stylesheet", href: APP_CSS }
        document::Link { rel: "stylesheet", href: DEFAULT_STYLE }
        document::Title { "dx-dnd · multi-list demo" }

        main { class: "app",
            header { class: "header",
                h1 { "dx-dnd" }
                p { "Drag cards between columns. Drops fire `on_drop`; the example mutates the source vec in place." }
            }

            DragDropArea::<i64> {
                class: "board",
                on_drop,
                for (list_id, items) in lists_snapshot.iter().cloned() {
                    section {
                        key: "col-{list_id}",
                        class: "column",
                        header { class: "column-header",
                            span { class: "column-title", "{list_id}" }
                            span { class: "column-count", "{items.len()}" }
                            button {
                                class: "add-btn",
                                onclick: {
                                    let list_id = list_id.clone();
                                    let mut add_to = add_to;
                                    move |_| add_to(list_id.clone())
                                },
                                "+"
                            }
                        }
                        DropList {
                            list_id: list_id.clone(),
                            count: items.len(),
                            class: "column-body",
                            for (idx, item) in items.iter().cloned().enumerate() {
                                Draggable::<i64> {
                                    key: "item-{item.id}",
                                    item_id: item.id,
                                    list_id: list_id.clone(),
                                    slot: idx,
                                    class: "card",
                                    "{item.label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "web")]
fn web_sys_log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

#[cfg(not(feature = "web"))]
fn web_sys_log(msg: &str) {
    eprintln!("{msg}");
}
