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

## Documentation

API docs are rustdoc-first — the crate root has a quick start, theming
notes, and a guide to the examples:

```sh
cargo doc --no-deps --open
```

(Once published to crates.io, the same docs will be on docs.rs, built for
`wasm32-unknown-unknown`.)

## Examples

One browser demo per component, each exercising every public parameter.
From `nix develop` (which provides a matching dioxus-cli 0.6.x, lld, and
wasm-bindgen-cli):

```sh
dx serve --example workspace --platform web
```

| example | shows |
| --- | --- |
| `workspace` | the whole workspace system: `use_workspace` + `PanelKind` + `LayoutBuilder`, floating mode (drag/resize/z-raise/traffic lights), tiling mode (red-light toggle, drag-header reorder, full-width panel via the `panel-<slug>` class), a panel that starts minimized in the dock, viewport clamping vs. stored geometry, localStorage persistence (`panel_kit_example_workspace`), the mobile stack, the `is_editing` shortcut gate, and a `tip_pos` tooltip overlay |
| `badge` | all ten `BadgeKind`s, every prop (`active`, `with_x`, `with_plus`, `small`, `override_color`, `accent_color`, both `BadgeClickKind`s, `emit_hover`) behind live toggles, an event log proving every `BadgeAction` variant fires, and a `tag_hue` FNV hue-spread row |
| `spinner` | `Spinner` with and without `label`, plus a live-editable label |
| `theming` | the documented retheme path: `:root` variable overrides layered after `panel_kit::CSS`, with three switchable presets |

`dx build --example <name> --platform web` produces the same app
statically under `target/dx/<name>/debug/web/public`.

Note: `Cargo.lock` pins `wasm-bindgen` to the exact version of nixpkgs'
`wasm-bindgen-cli` (dx refuses to bindgen with a mismatched CLI); keep the
two in lockstep when bumping the flake.

## Developing against a local checkout

In the consuming app's workspace `Cargo.toml`:

```toml
[patch."https://github.com/ocasazza/panel-kit"]
panel-kit = { path = "../../panel-kit" }
```
