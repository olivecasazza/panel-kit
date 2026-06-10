//! Theming demo — retheme panel-kit by overriding the `:root` CSS variables.
//!
//! Run with: `dx serve --example theming --platform web`
//! (dioxus-cli 0.6.x; provided by `nix develop`)
//!
//! What it demonstrates:
//! - The documented theming path: inject `panel_kit::CSS` once, then layer
//!   a stylesheet after it that overrides the `:root` variables (`--bg`,
//!   `--panel`, `--fg`, `--dim`, `--line`, `--line2`, `--inv-bg`,
//!   `--inv-fg`, `--accent`, `--red`, `--yellow`, `--green`, `--mono`).
//! - Three presets switchable at runtime: the built-in dark default, a
//!   light "paper" theme, and a CRT phosphor theme (which also swaps
//!   `--mono` to show the font variable).
//! - The whole chrome restyles: workspace panels, traffic lights, dock,
//!   badges, and the spinner all follow the variables.

use dioxus::prelude::*;
use panel_kit::badge::{Badge, BadgeAction, BadgeKind};
use panel_kit::{use_workspace, LayoutBuilder, PanelKind, PanelWin, Spinner, CSS};
use serde::{Deserialize, Serialize};

const DEMO_CSS: &str = "
.theme-pick { display: flex; gap: .4rem; margin-left: auto; }
.theme-pick button { background: var(--bg); color: var(--dim); border: 1px solid var(--line2);
  border-radius: 3px; padding: .15rem .5rem; font-size: .72rem; cursor: pointer; }
.theme-pick button.on { color: var(--fg); border-color: var(--accent); }
.swatches { display: flex; flex-wrap: wrap; gap: .5rem; align-items: center; }
";

/// Every themable variable, overridden to a warm light palette.
const THEME_PAPER: &str = ":root {
  --bg:    #f6f1e7;
  --panel: #fffdf6;
  --fg:    #211d16;
  --dim:   #8a8273;
  --line:  #e3dac6;
  --line2: #c9bda0;
  --inv-bg:#211d16;
  --inv-fg:#f6f1e7;
  --accent:#b3541e;
  --red:   #c4423a;
  --yellow:#b8860b;
  --green: #3e7d3a;
}";

/// Green-phosphor CRT — also swaps `--mono` to prove the font variable.
const THEME_CRT: &str = ":root {
  --bg:    #041204;
  --panel: #061a06;
  --fg:    #4ee04e;
  --dim:   #1f7a1f;
  --line:  #0c330c;
  --line2: #145214;
  --inv-bg:#4ee04e;
  --inv-fg:#041204;
  --accent:#9dff9d;
  --red:   #ff6b5f;
  --yellow:#e0d34e;
  --green: #4ee04e;
  --mono: 'Courier New', 'Courier', monospace;
}";

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Default,
    Paper,
    Crt,
}

impl Theme {
    fn label(self) -> &'static str {
        match self {
            Theme::Default => "dark (built-in)",
            Theme::Paper => "paper",
            Theme::Crt => "crt",
        }
    }
    /// The `:root` override block layered after `panel_kit::CSS`.
    fn css(self) -> &'static str {
        match self {
            Theme::Default => "", // no overrides — the library's own palette
            Theme::Paper => THEME_PAPER,
            Theme::Crt => THEME_CRT,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Panel {
    Swatches,
    About,
}

impl PanelKind for Panel {
    fn title(self) -> &'static str {
        match self {
            Panel::Swatches => "Swatches",
            Panel::About => "About",
        }
    }
}

fn default_layout() -> Vec<PanelWin<Panel>> {
    let mut b = LayoutBuilder::new();
    vec![
        b.at(Panel::Swatches, 16.0, 16.0, 520.0, 300.0),
        b.at(Panel::About, 556.0, 16.0, 380.0, 300.0),
    ]
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let ws = use_workspace("panel_kit_example_theming", default_layout);
    let mut theme = use_signal(|| Theme::Default);
    let theme_css = theme().css();

    let body = move |kind: Panel, _maximized: bool| -> Element {
        match kind {
            Panel::Swatches => rsx! {
                p { "Badges and the spinner pick the variables up too:" }
                div { class: "swatches",
                    Badge { field: "tag", value: "accent", kind: BadgeKind::Tag,
                        active: true, on_action: move |_: BadgeAction| {} }
                    Badge { field: "status", value: "green", kind: BadgeKind::Status,
                        on_action: move |_: BadgeAction| {} }
                    Badge { field: "entity", value: "yellow",
                        kind: BadgeKind::Entity { ty: None },
                        on_action: move |_: BadgeAction| {} }
                    Badge { field: "link", value: "red (unresolved)",
                        kind: BadgeKind::Wikilink { resolved: false, target: "x".to_string() },
                        on_action: move |_: BadgeAction| {} }
                    Spinner { label: "spinning in theme colours" }
                }
            },
            Panel::About => rsx! {
                p { "The stylesheet order is the whole mechanism:" }
                ol {
                    li { code { "style {{ {{panel_kit::CSS}} }}" } " — the library chrome" }
                    li { code { ":root {{ --bg: …; --accent: …; }}" }
                        " — your overrides, injected after it" }
                }
                p { "Switch presets in the top bar. The CRT preset also overrides "
                    code { "--mono" } ", the font stack variable." }
            },
        }
    };

    rsx! {
        style { {CSS} }
        style { {DEMO_CSS} }
        style { {theme_css} }
        div {
            class: ws.root_class(),
            onmousemove: move |e| ws.handle_mouse_move(&e),
            onmouseup: move |_| ws.handle_mouse_up(),
            header { class: "topbar",
                h1 { "panel-kit theming demo" }
                div { class: "theme-pick",
                    for t in [Theme::Default, Theme::Paper, Theme::Crt] {
                        button {
                            class: if theme() == t { "on" } else { "" },
                            onclick: move |_| theme.set(t),
                            {t.label()}
                        }
                    }
                }
            }
            {ws.render(body)}
            {ws.dock()}
        }
    }
}
