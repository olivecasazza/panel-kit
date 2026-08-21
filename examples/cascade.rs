//! Cascade demo + regression harness for the ticking-parent state reset.
//!
//! Run with: `dx serve --example cascade --platform web`
//! (dioxus-cli 0.6.x; provided by `nix develop`)
//!
//! What it demonstrates:
//! - CascadingDropdown over a robot -> campaign -> experiment tree, including
//!   root-level leaves (robots without policies).
//! - A parent that re-renders every 250 ms (the spot gym shell rerenders the
//!   settings panel on every telemetry tick): the open cascade must keep its
//!   descend path instead of resetting.
use dioxus::prelude::*;
use panel_kit::cascade::{CascadeAction, CascadeItem, CascadeState, CascadingDropdown};
use panel_kit::CSS;

fn main() {
    dioxus::launch(App);
}

fn items() -> Vec<CascadeItem> {
    let mut v = vec![];
    for campaign in ["spot-walk-pbt-v74", "spot-walk-pbt-v72"] {
        for i in 0..3 {
            v.push(CascadeItem {
                path: vec!["spot".into(), campaign.into()],
                value: format!("{campaign}-00{i}"),
                label: format!("00{i}"),
            });
        }
    }
    for robot in ["spider", "snake", "humanoid"] {
        v.push(CascadeItem {
            path: vec![],
            value: robot.into(),
            label: format!("{robot} (sandbox)"),
        });
    }
    v
}

#[component]
fn App() -> Element {
    // Parent render storm: 4 ticks/second, like the gym shell's telemetry.
    let mut tick = use_signal(|| 0u32);
    use_hook(move || {
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(250).await;
                tick += 1;
            }
        });
    });

    let mut cascade_state = use_signal(CascadeState::default);
    let mut selected = use_signal(|| "(none)".to_string());

    rsx! {
        style { {CSS} }
        div { style: "padding:2rem; font-family:monospace;",
            h1 { "cascade demo (tick {tick})" }
            CascadingDropdown {
                items: items(),
                state: cascade_state,
                selected_label: selected(),
                placeholder: "pick a robot / policy".to_string(),
                on_action: move |a: CascadeAction| {
                    if let CascadeAction::Select { path, value } = a {
                        selected.set(format!("{path:?} -> {value}"));
                    }
                },
            }
            p { "selected: {selected}" }
            button {
                r#type: "button",
                onclick: move |_| cascade_state.set(CascadeState::open()),
                "open programmatically"
            }
        }
    }
}
