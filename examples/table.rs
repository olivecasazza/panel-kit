//! Table demo: long cell values, host-owned selection, dense mode.
//!
//! Run with: `dx serve --example table --platform web`
//! (dioxus-cli 0.6.x; provided by `nix develop`)

use dioxus::prelude::*;
use panel_kit::table::Table;
use panel_kit::CSS;

fn main() {
    dioxus::launch(App);
}

fn rows() -> Vec<Vec<String>> {
    (0..20)
        .map(|i| {
            vec![
                format!("spot-walk-pbt-v74-{i:03}"),
                format!("{:.2}", 0.72 - i as f32 * 0.03),
                if i % 3 == 0 {
                    "real trot with a very long descriptive label that would \
                     previously have been unreadable in a clipped cell"
                        .to_string()
                } else {
                    "velocity tracker".to_string()
                },
                format!("{}", 1_000_000 + i * 37_000),
                if i % 2 == 0 { "kept" } else { "reverted" }.to_string(),
            ]
        })
        .collect()
}

#[component]
fn App() -> Element {
    let mut selected = use_signal(|| None::<usize>);
    let mut dense = use_signal(|| false);

    rsx! {
        style { {CSS} }
        div { style: "padding:2rem; max-width:56rem; font-family:monospace;",
            h3 { "panel-kit table" }
            label {
                input {
                    r#type: "checkbox",
                    checked: dense(),
                    onchange: move |ev| dense.set(ev.checked()),
                }
                " dense"
            }
            p {
                match selected() {
                    Some(i) => format!("selected row: {i}"),
                    None => "click a row".to_string(),
                }
            }
            Table {
                columns: vec![
                    "policy".into(),
                    "reward".into(),
                    "notes".into(),
                    "steps".into(),
                    "outcome".into(),
                ],
                rows: rows(),
                selected: selected(),
                dense: dense(),
                on_row_click: move |i: usize| selected.set(Some(i)),
            }
        }
    }
}
