//! Bevy canvas panel component.
//!
//! Provides [`BevyCanvas`], a Dioxus component that dynamically loads and mounts
//! a Bevy WASM module into a canvas element. The module must export:
//! - `default()` — async init function (wasm-pack standard)
//! - `WebHandle` class with `new()` and `start(canvas)` methods
//!
//! After mounting, the handle is stored in a global registry accessible via
//! [`get_bevy_handle`] so external components can control the running demo.
//!
//! # Example
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use panel_kit::BevyCanvas;
//!
//! fn my_panel() -> Element {
//!     rsx! {
//!         BevyCanvas {
//!             module_path: "/wasm/flock/pkg/flock.js",
//!             canvas_id: "flock-canvas",
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

thread_local! {
    /// Registry of running bevy handles, keyed by canvas_id
    static BEVY_HANDLES: RefCell<HashMap<String, JsValue>> = RefCell::new(HashMap::new());
}

/// Get a reference to a running bevy handle by canvas ID.
/// Returns None if no handle exists for that canvas.
pub fn get_bevy_handle(canvas_id: &str) -> Option<JsValue> {
    BEVY_HANDLES.with(|h| h.borrow().get(canvas_id).cloned())
}

/// Store a bevy handle in the registry
fn store_bevy_handle(canvas_id: &str, handle: JsValue) {
    BEVY_HANDLES.with(|h| {
        h.borrow_mut().insert(canvas_id.to_string(), handle);
    });
}

/// Remove a bevy handle from the registry (for cleanup)
#[allow(dead_code)]
fn remove_bevy_handle(canvas_id: &str) {
    BEVY_HANDLES.with(|h| {
        h.borrow_mut().remove(canvas_id);
    });
}

/// CSS for the bevy canvas component — inject with `style { {BEVY_CSS} }` or
/// include in your app's stylesheet.
pub const BEVY_CSS: &str = r#"
.bevy-canvas-container { width: 100%; height: 100%; position: relative; overflow: hidden; }
.bevy-canvas { width: 100%; height: 100%; display: block; touch-action: none; position: relative; z-index: 1; }
.bevy-canvas-loading {
    position: absolute; inset: 0; z-index: 0;
    display: flex; align-items: center; justify-content: center;
    color: var(--dim, #7a7a7a); font-size: 12px;
    pointer-events: none;
}
.bevy-canvas-error {
    position: absolute; inset: 0; z-index: 2;
    display: flex; align-items: center; justify-content: center;
    color: var(--red, #ff5f56); font-size: 12px; padding: 8px; text-align: center;
}
"#;

/// State of the bevy module loading process
#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Running,
    Error(String),
}

/// A Dioxus component that loads and mounts a Bevy WASM module.
///
/// The module is loaded dynamically via ES module import, so it must be
/// served as a separate WASM bundle (built with `wasm-pack build --target web`).
#[component]
pub fn BevyCanvas(
    /// Path to the JS module (e.g. "/wasm/flock/pkg/flock.js")
    module_path: String,
    /// Canvas element ID (must be unique if multiple canvases)
    #[props(default = "bevy-canvas".to_string())]
    canvas_id: String,
    /// Optional loading text
    #[props(default = "loading...".to_string())]
    loading_text: String,
) -> Element {
    let mut state = use_signal(|| LoadState::Loading);
    let canvas_id_clone = canvas_id.clone();
    let module_path_clone = module_path.clone();

    // Load the wasm module after the canvas is mounted
    use_effect(move || {
        let canvas_id = canvas_id_clone.clone();
        let module_path = module_path_clone.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match load_and_start_bevy(&module_path, &canvas_id).await {
                Ok(()) => state.set(LoadState::Running),
                Err(e) => {
                    let msg = format!("{:?}", e);
                    web_sys::console::error_1(&msg.clone().into());
                    state.set(LoadState::Error(msg));
                }
            }
        });
    });

    let current_state = state.read().clone();

    rsx! {
        div { class: "bevy-canvas-container",
            canvas {
                id: "{canvas_id}",
                class: "bevy-canvas",
            }
            match current_state {
                LoadState::Loading => rsx! {
                    div { class: "bevy-canvas-loading", "{loading_text}" }
                },
                LoadState::Error(msg) => rsx! {
                    div { class: "bevy-canvas-error", "{msg}" }
                },
                LoadState::Running => rsx! {},
            }
        }
    }
}

/// Load a bevy WASM module and start it on the specified canvas.
/// Note: Bevy's run() takes over the event loop and never returns, so we
/// consider it "running" once start() is called (don't await its promise).
async fn load_and_start_bevy(module_path: &str, canvas_id: &str) -> Result<(), JsValue> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    // Wait a tick for the canvas to be in the DOM
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
            .unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(promise).await?;

    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| format!("canvas '{}' not found", canvas_id))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    // Dynamic import of the wasm module
    let import_promise = js_sys::eval(&format!(r#"import("{}")"#, module_path))?;
    let module =
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(import_promise)).await?;

    // Call default() to init wasm
    let default_fn = js_sys::Reflect::get(&module, &"default".into())?;
    let init_promise = js_sys::Function::from(default_fn).call0(&JsValue::NULL)?;
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(init_promise)).await?;

    // Create WebHandle and start - don't await, bevy's run() never returns
    let web_handle_class = js_sys::Reflect::get(&module, &"WebHandle".into())?;
    let handle = js_sys::Reflect::construct(
        &js_sys::Function::from(web_handle_class),
        &js_sys::Array::new(),
    )?;

    // Store handle in registry before starting
    store_bevy_handle(canvas_id, handle.clone());

    let start_fn = js_sys::Reflect::get(&handle, &"start".into())?;
    // Fire and forget - bevy takes over the event loop
    let _ = js_sys::Function::from(start_fn).call1(&handle, &canvas)?;

    Ok(())
}
