//! Terminal twin of the web `workspace` example: three panels you can
//! drag, resize, reorder, minimize to the dock, and maximize — with the
//! layout persisted across runs.
//!
//! Run: `cargo run -p panel-kit-tui --example workspace`
//! Mouse: drag headers to move (floating) / reorder (tiling), drag the ◢
//! grip to resize (span-snapped in tiling), click ⊞/❐ to flip mode, − to
//! minimize, ⤢ to maximize, dock chips to restore. `q` quits.

use std::time::Duration;

use panel_kit_core::{LayoutBuilder, PanelKind, PanelWin};
use panel_kit_tui::TuiWorkspace;
use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use ratatui::crossterm::execute;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Panel {
    Logs,
    Stats,
    Help,
}

impl PanelKind for Panel {
    fn title(self) -> &'static str {
        match self {
            Panel::Logs => "Logs",
            Panel::Stats => "Stats",
            Panel::Help => "Help",
        }
    }
}

fn defaults() -> Vec<PanelWin<Panel>> {
    let mut b = LayoutBuilder::new();
    vec![
        b.at(Panel::Logs, 2.0, 1.0, 52.0, 14.0).with_tile(2, 2),
        b.at(Panel::Stats, 56.0, 1.0, 38.0, 10.0),
        b.at(Panel::Help, 20.0, 9.0, 46.0, 9.0),
    ]
}

fn main() -> std::io::Result<()> {
    let store = std::env::temp_dir().join("panel-kit-tui-demo.json");
    let mut ws = TuiWorkspace::new(Some(store), defaults);

    let mut terminal = ratatui::init();
    let _ = execute!(std::io::stdout(), EnableMouseCapture);
    let mut frames: u64 = 0;
    loop {
        frames += 1;
        terminal.draw(|f| {
            ws.render(f, f.area(), &mut |f, rect, kind, maximized| {
                let text = match kind {
                    Panel::Logs => format!(
                        "frame {frames}\nthe same panel state machine\nthe web shell renders to DOM\n…rendered in terminal cells"
                    ),
                    Panel::Stats => format!("panels: 3\nmaximized: {maximized}\nlayout: persisted to /tmp"),
                    Panel::Help => "drag headers · ◢ resizes\n⊞ mode · − minimize · ⤢ maximize\nq quits".to_string(),
                };
                f.render_widget(
                    Paragraph::new(text).style(Style::default().fg(Color::Gray)),
                    rect,
                );
            });
        })?;
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) if k.code == KeyCode::Char('q') => break,
                Event::Mouse(m) => ws.handle_mouse(m),
                _ => {}
            }
        }
    }
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    Ok(())
}
