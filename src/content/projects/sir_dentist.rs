//! Project: Sir Dentist, a 2D Unity game.

use crate::content::{bullet, heading, meta};
use ratzilla::ratatui::text::{Line, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Sir Dentist"),
        meta("Game Design, Tufts University | Oct. 2023 - Dec. 2023"),
        Line::raw(""),
        bullet("Developed a 2D Unity game in C# with a team, using GitHub for version control and project management"),
        bullet("Led team meetings, delegated tasks, and coordinated between developers to hit project milestones on time"),
    ])
}
