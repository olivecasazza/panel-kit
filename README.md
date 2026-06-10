# panel-kit

Generic Dioxus panel-workspace library. Every view is a panel you can
move/resize/minimize/maximize, with floating (free placement) and tiling
(auto grid) workspace modes, macOS-style traffic lights, tiling
drag-to-reorder, a minimized-panel dock strip, and layout persistence to
localStorage. Includes a reusable `Badge` chip component and `Spinner`.

Factored out of [jump-cannon](https://github.com/ocasazza/jump-cannon) and
apple-notes-ocr-flow, which both consume it as a git dependency:

```toml
[dependencies]
panel-kit = { git = "https://github.com/ocasazza/panel-kit" }
```

## Usage

The app supplies two things: a `PanelKind` impl (an enum of its panels) and a
body-render callback. Everything else — geometry, z-order, drag state,
viewport clamping, the mobile breakpoint, persistence — lives here.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Panel { Graph, Inspector }
impl panel_kit::PanelKind for Panel {
    fn title(self) -> &'static str { /* … */ }
}

let ws = panel_kit::use_workspace("myapp_layout", default_layout);
rsx! {
    style { {panel_kit::CSS} }
    div { class: ws.root_class(),
        onmousemove: move |e| ws.handle_mouse_move(&e),
        onmouseup: move |_| ws.handle_mouse_up(),
        header { class: "topbar", /* app-specific */ }
        {ws.render(|kind, maximized| rsx! { /* panel body for `kind` */ })}
        {ws.dock()}
    }
}
```

Inject `panel_kit::CSS` once at the app root, then layer app-specific styles
after it; override the `:root` variables to retheme.

## Developing against a local checkout

In the consuming app's workspace `Cargo.toml`:

```toml
[patch."https://github.com/ocasazza/panel-kit"]
panel-kit = { path = "../../panel-kit" }
```
