//! The "Experience" page.

use crate::content::{bullet, heading, section};
use ratzilla::ratatui::text::{Line, Span, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Experience"),
        Line::raw(""),
        // Tufts: Teaching Fellow
        section("Tufts University"),
        Line::from(vec![
            Span::raw("Programming Languages Teaching Fellow"),
            Span::raw(" | "),
            Span::raw("Medford, MA"),
            Span::raw(" | "),
            Span::raw("Sept. 2025 - Present"),
        ]),
        Line::raw(""),
        bullet(
            "Review and refine the CS 105 programming languages curriculum in collaboration with faculty",
        ),
        bullet(
            "Mentor new teaching assistants on effective debugging strategies and course instruction practices",
        ),
        bullet(
            "Lead all recitations for ~20 students on operational semantics, type inference, and continuations",
        ),
        Line::raw(""),
        Line::raw(""),
        // Tufts: Teaching Assistant
        section("Tufts University"),
        Line::from(vec![
            Span::raw("Programming Languages Teaching Assistant"),
            Span::raw(" | "),
            Span::raw("Medford, MA"),
            Span::raw(" | "),
            Span::raw("Sept. 2024 - May 2025"),
        ]),
        Line::raw(""),
        bullet(
            "Taught students flexible programming skills that are applicable to any programming language",
        ),
        bullet(
            "Guided students in applying mathematical foundations to analyze programming language structure",
        ),
        Line::raw(""),
        Line::raw(""),
        // Mapunawai
        section("Mapunawai"),
        Line::from(vec![
            Span::raw("Community Digital Navigator"),
            Span::raw(" | "),
            Span::raw("Kauai, HI"),
            Span::raw(" | "),
            Span::raw("Jun. 2026 - Aug. 2026"),
        ]),
        Line::raw(""),
        bullet(
            "Guided ~30 elderly patrons weekly through smartphone, tablet, and laptop use and troubleshooting",
        ),
        bullet(
            "Taught patrons how to use cloud services, the most frequent point of confusion among learners",
        ),
        Line::raw(""),
        Line::raw(""),
        // Security & Privacy Lab
        section("Tufts Security & Privacy Lab"),
        Line::from(vec![
            Span::raw("Research Assistant"),
            Span::raw(" | "),
            Span::raw("Medford, MA"),
            Span::raw(" | "),
            Span::raw("Sept. 2023 - May 2026"),
        ]),
        Line::raw(""),
        bullet(
            "Built Docker & Python tooling to run C++ secure two-party computation experiments using EMP-toolkit",
        ),
        bullet(
            "Built a Python CLI that parses SQL into ASTs and answers PostgreSQL queries with differential privacy",
        ),
        Line::raw(""),
        Line::raw(""),
        // PMRF
        section("Pacific Missile Range Facility"),
        Line::from(vec![
            Span::raw("Communication Systems Intern"),
            Span::raw(" | "),
            Span::raw("Kekaha, HI"),
            Span::raw(" | "),
            Span::raw("Jun. 2025 - Aug. 2025"),
        ]),
        Line::raw(""),
        bullet(
            "Authored a standard operating procedure for RF equipment, enabling consistent and safe use by staff",
        ),
        bullet(
            "Processed 13GB of RF data using Python (NumPy, pandas, matplotlib) to generate graphs for reports",
        ),
    ])
}
