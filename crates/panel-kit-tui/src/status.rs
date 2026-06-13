//! Status-dot component — a colored `●` for state readouts in tables and
//! lists. Callers map their own domain state to a [`Color`] (the meaning of
//! each color is app-specific) and use these helpers to render it
//! consistently.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// A colored status dot `●`.
pub fn dot(color: Color) -> Span<'static> {
    Span::styled("●", Style::default().fg(color))
}

/// A `Style` whose foreground is `color` — convenience for styling a table
/// cell to match its [`dot`].
pub fn style(color: Color) -> Style {
    Style::default().fg(color)
}

/// A `● label` line: the dot and the label both in `color`.
pub fn labeled(color: Color, label: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        dot(color),
        Span::raw(" "),
        Span::styled(label.into(), Style::default().fg(color)),
    ])
}
