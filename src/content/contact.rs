//! The "Contact" page.

use crate::content::{heading, link, meta};
use ratzilla::ratatui::text::{Line, Span, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line. `link(text, url)` makes a
/// clickable link.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Contact"),
        Line::raw(""),
        Line::raw("Please feel free to reach out to me with any of the methods below."),
        Line::raw(""),
        meta("Note that the email provided below is an alias that forwards to my real proton email"),
        Line::raw(""),
        Line::from(vec![
            Span::raw("Email:    "),
            link(
                "portfolio.snowdrop482@passinbox.com",
                "mailto:portfolio.snowdrop482@passinbox.com",
            ),
        ]),
        Line::from(vec![
            Span::raw("LinkedIn: "),
            link(
                "linkedin.com/in/joe-ansel-puplava",
                "https://www.linkedin.com/in/joe-ansel-puplava/",
            ),
        ]),
        Line::from(vec![
            Span::raw("GitHub:   "),
            link(
                "github.com/JoeAnselPuplava",
                "https://github.com/JoeAnselPuplava",
            ),
        ]),
    ])
}
