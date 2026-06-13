//! Themed table component. A thin wrapper over ratatui's [`Table`] that
//! applies the panel-kit chrome convention — a dim header row over the data —
//! while leaving cell construction to the caller, so individual cells can be
//! styled (e.g. with [`crate::status`]) for status coloring.
//!
//! ```ignore
//! use ratatui::layout::Constraint;
//! use ratatui::widgets::{Cell, Row};
//! panel_kit_tui::table::table(
//!     f, rect, &theme,
//!     &["task", "status"],
//!     &[Constraint::Length(20), Constraint::Length(8)],
//!     rows.into_iter().map(|t| Row::new(vec![
//!         Cell::from(t.name),
//!         Cell::from(t.status).style(panel_kit_tui::status::style(color)),
//!     ])).collect(),
//! );
//! ```

use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Row, Table};
use ratatui::Frame;

use crate::Theme;

/// Render a themed table into `area`: a dim `header` row over `rows`, with
/// column `widths`. The caller builds the [`Row`]s so cells keep full control
/// over their own styling.
pub fn table(
    f: &mut Frame,
    area: Rect,
    t: &Theme,
    header: &[&str],
    widths: &[Constraint],
    rows: Vec<Row<'_>>,
) {
    let header = Row::new(header.iter().copied()).style(Style::default().fg(t.dim));
    f.render_widget(
        Table::new(rows, widths.iter().copied()).header(header),
        area,
    );
}
