//! The "About" page.

use crate::content::heading;
use ratzilla::ratatui::text::{Line, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("About"),
        Line::raw(""),
        // Line::raw(
        //     "Write a short introduction here: who you are, what you do and what you are interested in.",
        // ),
        Line::raw(
            "Hello! My name is Joe-Ansel Puplava and welcome to my website. I'm a Computer Engineering Master's student at Tufts University interested in compilers, embedded systems, embedded security, and video games (mostly playing them).",
        ),
        Line::raw(""),
        Line::raw(
            "On my website you can find out what projects I've worked on, my work experience, and how to contact me. Have fun exploring!",
        ),
    ])
}
