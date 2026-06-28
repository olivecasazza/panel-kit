//! Code / manifest editor panel component.
//!
//! Provides [`IdePanel`], a generic, dependency-light code editor: a
//! monospace `<textarea>` with a line-number gutter and a language label. It
//! is the Dioxus generalization of the Iced manifest editor in
//! athena-console — the same semantics (a labelled document, a language/kind
//! tag, and a read-only mode) ported to the DOM, with no syntax-highlighting
//! dependency.
//!
//! The gutter is kept scroll-synced with the textarea through an `onscroll`
//! handler (no JS library), and the line count is derived from the value's
//! newlines. In read-only mode the textarea carries the `readonly` attribute
//! and the editor is visually muted; otherwise edits flow out through
//! [`on_change`](IdePanelProps::on_change) on every input.
//!
//! Styling comes from the `.ide-panel*` rules in [`crate::CSS`], themed off
//! the same `:root` variables as the rest of the chrome (`--bg`, `--fg`,
//! `--line2`, `--accent`, `--mono`).
//!
//! # Example
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use panel_kit::IdePanel;
//!
//! fn editor(mut doc: Signal<String>) -> Element {
//!     rsx! {
//!         IdePanel {
//!             value: doc(),
//!             language: "yaml",
//!             on_change: move |next: String| doc.set(next),
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide counter for unique textarea ids, so the gutter scroll-sync can
/// find *its* textarea by id even with several [`IdePanel`]s on the page.
static IDE_SEQ: AtomicU64 = AtomicU64::new(0);

/// A monospace code/manifest editor with a line-number gutter and language
/// label.
///
/// Mirrors the read-only / language-kind semantics of the Iced manifest
/// editor it generalizes. A styled `<textarea>` plus a scroll-synced gutter;
/// no syntax highlighting and no JS dependency. See the [module
/// docs](crate::ide) and [`crate::CSS`] for styling.
#[component]
pub fn IdePanel(
    /// Current editor text. This is a controlled value — render it from your
    /// own state and update that state in [`on_change`](IdePanelProps::on_change).
    value: String,
    /// Language tag shown in the header label and used as the gutter/editor
    /// `data-language` attribute (e.g. `"yaml"`, `"json"`). When `None` the
    /// label reads `"text"`.
    #[props(default)]
    language: Option<String>,
    /// Fired on every edit with the textarea's full new value. Ignored while
    /// [`read_only`](IdePanelProps::read_only) is set.
    on_change: EventHandler<String>,
    /// When true, the textarea is `readonly` and visually muted; edits are
    /// suppressed. Mirrors the Iced editor's read-only manifest view.
    #[props(default)]
    read_only: bool,
    /// Optional document title shown left of the language label (e.g. a file
    /// name). Omitted when `None`.
    #[props(default)]
    title: Option<String>,
) -> Element {
    // Gutter scroll offset, in CSS px, mirrored from the textarea's scrollTop
    // so the line numbers track the text as it scrolls.
    let mut scroll_top = use_signal(|| 0.0_f64);
    // Stable per-instance id, assigned once, so onscroll can read this
    // textarea's scrollTop by id (robust to multiple editors on the page).
    let textarea_id =
        use_hook(|| format!("ide-textarea-{}", IDE_SEQ.fetch_add(1, Ordering::Relaxed)));

    let lang = language.clone().unwrap_or_else(|| "text".to_string());
    // Line count drives the gutter; always at least one row, and a trailing
    // newline adds the empty final line the textarea shows.
    let line_count = value.bytes().filter(|&b| b == b'\n').count() + 1;

    let mut class = String::from("ide-panel");
    if read_only {
        class.push_str(" read-only");
    }

    let gutter_style = format!("transform:translateY(-{}px);", scroll_top());

    rsx! {
        div { class: "{class}", "data-language": "{lang}",
            header { class: "ide-head",
                if let Some(title) = title.as_ref() {
                    span { class: "ide-title", title: "{title}", "{title}" }
                }
                span { class: "ide-spacer" }
                span { class: "ide-lang", "{lang}" }
                if read_only {
                    span { class: "ide-readonly", "read-only" }
                }
            }
            div { class: "ide-body",
                div { class: "ide-gutter",
                    div { class: "ide-gutter-inner", style: "{gutter_style}",
                        for n in 1..=line_count {
                            div { class: "ide-line-no", "{n}" }
                        }
                    }
                }
                textarea {
                    id: "{textarea_id}",
                    class: "ide-textarea",
                    spellcheck: "false",
                    autocomplete: "off",
                    autocapitalize: "off",
                    "autocorrect": "off",
                    wrap: "off",
                    readonly: read_only,
                    value: "{value}",
                    oninput: move |e| {
                        if !read_only {
                            on_change.call(e.value());
                        }
                    },
                    onscroll: {
                        let textarea_id = textarea_id.clone();
                        move |_| {
                            // Dioxus scroll events don't carry scrollTop, so
                            // read it off this textarea (by id) through web-sys
                            // and mirror it onto the gutter via translateY.
                            if let Some(top) = element_scroll_top(&textarea_id) {
                                scroll_top.set(top);
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Read `scrollTop` off the element with `id`, used to keep the gutter aligned
/// with the textarea during a scroll. Returns `None` outside a browser or when
/// no such element exists yet.
fn element_scroll_top(id: &str) -> Option<f64> {
    let el = web_sys::window()?.document()?.get_element_by_id(id)?;
    Some(el.scroll_top() as f64)
}
