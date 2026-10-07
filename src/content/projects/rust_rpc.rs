//! Project: Rust RPC parallel optimizations (research, in progress).

use crate::content::{bullet, heading, meta};
use ratzilla::ratatui::text::{Line, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Rust RPC Parallel Optimizations"),
        meta("Tufts University | May 2026 - Present"),
        Line::raw(""),
        Line::raw(
            "A research project that is currently in the design phase.",
        ),
        Line::raw(""),
        bullet("Designing a Rust tool that detects data-independent code and lowers it to a parallel IR for RPC execution"),
    ])
}
