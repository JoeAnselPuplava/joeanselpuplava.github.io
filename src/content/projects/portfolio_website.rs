//! Project: this portfolio website.

use crate::content::{bullet, heading, link, section};
use ratzilla::ratatui::text::{Line, Span, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Portfolio Website"),
        Line::from(vec![
            Span::raw("Repo: "),
            link(
                "github.com/JoeAnselPuplava/joeanselpuplava.github.io",
                "https://github.com/JoeAnselPuplava/joeanselpuplava.github.io",
            ),
        ]),
        Line::raw(""),
        Line::raw(
            "A terminal-style portfolio site written in Rust. It is drawn with Ratatui, runs in the browser as WebAssembly through ratzilla, and is built with trunk.",
        ),
        Line::raw(""),
        section("Features"),
        bullet("Vim-style navigation: j/k, gg/G, counts like 5j, Ctrl-d/u"),
        bullet("Nested menus you can open with Space and leave with Backspace"),
        bullet("Mouse support: click, double-click and scroll wheel"),
        bullet("An animated rain background (tui-rain)"),
    ])
}
