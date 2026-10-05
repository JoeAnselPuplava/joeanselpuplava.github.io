//! Home screen: title, menu, content pane and status bar.

use std::time::Duration;

use ratzilla::ratatui::{
    Frame,
    layout::{
        Constraint::{self, Percentage, Ratio},
        Layout, Margin, Rect, Spacing,
    },
    style::{Color, Style},
    symbols::merge::MergeStrategy,
    text::{Line, Span, Text},
    widgets::{
        Block, Clear, List, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};

use crate::app::{App, Entry, Pane};
/// Added after a directory's name in the menu. Try " ▸", " ›" or " ⏵".
const DIR_MARKER: &str = "/";

/// An entry's name as listed: directories get DIR_MARKER.
fn label(entry: &Entry) -> String {
    if entry.is_dir() {
        format!("{}{DIR_MARKER}", entry.title)
    } else {
        entry.title.to_string()
    }
}

/// Where each part of the home screen sits on the terminal grid.
pub struct Areas {
    pub whole: Rect, // everything below, used to clear the rain
    pub title: Rect,
    pub menu: Rect,
    pub content: Rect,
    pub status: Rect,
}

/// Splits the screen into the home screen's areas. Shared by `draw` and the
/// mouse code in `update.rs` so clicks hit exactly what is drawn.
pub fn layout(screen: Rect) -> Areas {
    use Constraint::{Fill, Length, Min};
    // Use the middle 70% of the screen; the rain shows around it.
    let area = screen.centered(Percentage(70), Percentage(70));

    // Rows: title, main area, status bar. Overlap(1) makes neighbouring
    // borders share a line so merge_borders can join them.
    let vertical = Layout::vertical([Length(2), Min(0), Length(4)]);
    let [title_area, main_area, status_area] = vertical.spacing(Spacing::Overlap(1)).areas(area);

    // Columns: menu (1/5 of the width) and content (4/5).
    let horizontal = Layout::horizontal([Fill(1), Fill(4)]);
    let [left_area, right_area] = horizontal.spacing(Spacing::Overlap(1)).areas(main_area);

    Areas {
        whole: area,
        title: title_area,
        menu: left_area,
        content: right_area,
        status: status_area,
    }
}

/// The menu's title, e.g. "Home/Projects". Every part except the last (where
/// we already are) is underlined to show it can be clicked to go back.
fn breadcrumb(app: &App) -> Line<'static> {
    let parts = app.breadcrumb();
    let last = parts.len() - 1;
    let spans = parts.into_iter().enumerate().map(|(i, part)| {
        if i < last {
            Span::styled(part, Style::new().underlined())
        } else {
            Span::raw(part)
        }
    });
    Line::from_iter(spans)
}

/// Text for an entry. Placeholder for now: replace with your real pages.
/// Each `Line` is a paragraph; long ones wrap to fit the content pane.
/// Use `Span::styled` to color part of a line.
pub fn page_text(entry: &Entry) -> Text<'static> {
    // A directory previews what's inside it.
    if entry.is_dir() {
        let mut lines = vec![
            Line::raw(label(entry)),
            Line::raw(""),
            Line::from(vec![
                Span::raw(
                    "Press Space or Enter to open. To come back to the home page \
                     press BackSpace or click on ",
                ),
                Span::styled("Home/", Style::new().fg(Color::LightCyan).underlined()),
                Span::raw(" at the top left."),
            ]),
            Line::raw(""),
        ];
        for child in entry.children {
            lines.push(Line::raw(format!("  {}", label(child))));
        }
        return Text::from(lines);
    }
    let mut lines = vec![
        Line::raw(format!("Content for {}", entry.title)),
        Line::raw(""),
    ];
    for n in 1..=40 {
        lines.push(Line::raw(format!("Placeholder line {n}")));
    }
    Text::from(lines)
}

/// The content pane's text, wrapped at word boundaries. `trim: false` keeps
/// leading spaces, so indented lines stay indented.
fn content_paragraph(entry: &Entry) -> Paragraph<'static> {
    Paragraph::new(page_text(entry)).wrap(Wrap { trim: false })
}

/// How far the content pane can scroll: the lines (after wrapping) that
/// don't fit in the pane. 0 means everything fits, so it can't scroll at all.
pub fn max_scroll(entry: &Entry, areas: &Areas) -> u16 {
    let inner = areas.content.inner(Margin::new(1, 1)); // inside the border
    let lines = content_paragraph(entry).line_count(inner.width);
    lines
        .saturating_sub(inner.height as usize)
        .try_into()
        .unwrap_or(u16::MAX)
}

/// Draws the home screen from the current `App` state.
pub fn draw(frame: &mut Frame, app: &App, elapsed: Duration) {
    let Areas {
        whole: area,
        title: title_area,
        menu: left_area,
        content: right_area,
        status: status_area,
    } = layout(frame.area());

    // rain first so the home screen sits on top of it.
    if !app.no_rain {
        crate::rain::view(frame, elapsed);
    }

    // Erase the rain behind the home screen.
    frame.render_widget(Clear, area);
    frame.render_widget(
        Text::raw("Joe-Ansel Puplava's Portfolio").centered(),
        title_area.centered_vertically(Ratio(1, 2)),
    );

    // Yellow border on the pane that has focus.
    let focused = |p| {
        if app.focus == p {
            Color::LightCyan
        } else {
            Color::Reset
        }
    };

    // Left pane: menu with the selected item highlighted
    let list = List::new(app.menu().iter().map(label))
        .block(
            Block::bordered()
                .title(breadcrumb(app))
                .merge_borders(MergeStrategy::Exact)
                .border_style(focused(Pane::Menu)),
        )
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold().black().bg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, left_area, &mut state);

    // Right pane: content for the selected item, scrolled
    let content = content_paragraph(app.current())
        .block(
            Block::bordered()
                .merge_borders(MergeStrategy::Exact)
                .border_style(focused(Pane::Content)),
        )
        .scroll((app.scroll, 0));
    frame.render_widget(content, right_area);

    // Scroll indicator on the content pane's right border, only when there is
    // more text than fits. The thumb's size shows how much of the page is visible.
    if app.max_scroll > 0 {
        let visible = right_area.height.saturating_sub(2) as usize;
        let mut scrollbar_state = ScrollbarState::new(app.max_scroll as usize + 1)
            .position(app.scroll as usize)
            .viewport_content_length(visible);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            // Stay between the top and bottom corners of the border.
            right_area.inner(Margin::new(0, 1)),
            &mut scrollbar_state,
        );
    }

    // Status bar: the keys that do something right now. On the right of its
    // border, like vim's showcmd, the count or "g" being typed.
    let typed = format!(
        "{}{}",
        app.count.map(|c| c.to_string()).unwrap_or_default(),
        app.pending.map(String::from).unwrap_or_default()
    );
    frame.render_widget(
        Paragraph::new(key_hints(app))
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title("Hints")
                    .title_top(Line::raw(typed).right_aligned())
                    .merge_borders(MergeStrategy::Exact),
            ),
        status_area,
    );
}

/// Key binds for the status bar, depending on which pane has focus, whether
/// the highlighted entry is a directory, whether we're inside one, and
/// whether the content can scroll, plus a note on its own line below.
/// Keep in sync with `update::update`.
fn key_hints(app: &App) -> Text<'static> {
    let mut hints: Vec<(&str, &str)> = Vec::new();
    match app.focus {
        Pane::Menu => {
            hints.push(("j/k", "up/down"));
            hints.push(("gg/G", "first/last"));
            hints.push(("l", "focus content"));
        }
        Pane::Content => {
            if app.max_scroll > 0 {
                hints.push(("j/k", "scroll"));
                hints.push(("Ctrl-d/u", "half page"));
                hints.push(("gg/G", "top/bottom"));
            }
            hints.push(("h", "focus menu"));
        }
    }

    hints.push(("x", if app.no_rain { "rain on" } else { "rain off" }));

    if app.current().is_dir() {
        hints.push(("Space/Enter", "open"));
    }
    if !app.path.is_empty() {
        hints.push(("Backspace", "back"));
    }

    // Key in LightCyan, then what it does, with a gap between hints.
    let mut spans = Vec::new();
    for (i, (key, action)) in hints.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(Span::styled(key, Style::new().fg(Color::LightCyan).bold()));
        spans.push(Span::raw(format!(" {action}")));
    }

    // A `Text` is a list of lines, so the note always starts on the next line.
    let note = Line::from(vec![
        Span::styled("Note:", Style::new().fg(Color::LightCyan).bold()),
        Span::raw(" Mouse actions also work!"),
    ]);
    Text::from(vec![Line::from(spans), note])
}
