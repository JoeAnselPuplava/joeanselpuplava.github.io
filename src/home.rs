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

use crate::app::{App, Entry, HOME_LABEL, Pane};
use crate::content::{Target, child_link};
use crate::theme::Theme;
use crate::update::{self, Clickable, Hovered};
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
/// mouse code in `update.rs` so clicks hit exactly what is drawn. Takes the
/// app because the hints box's height depends on which hints are showing.
pub fn layout(screen: Rect, app: &App) -> Areas {
    use Constraint::{Fill, Length, Min};
    // Use the middle 70% of the screen; the rain shows around it.
    let area = screen.centered(Percentage(70), Percentage(70));

    // Rows: title, main area, status bar. Overlap(1) makes neighbouring
    // borders share a line so merge_borders can join them. The status bar
    // fits the key hints (2 rows when they wrap on a narrower screen), plus
    // a row for the note or mouse hint below them, plus its 2 borders.
    let status_rows = key_hint_rows(app, area.width) + 3;
    let vertical = Layout::vertical([Length(2), Min(0), Length(status_rows)]);
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
                    "Press Space or Enter to open, or click one of the links below. \
                     To come back to the home page press BackSpace or click on ",
                ),
                Span::styled("Home/", Style::new().fg(Color::LightCyan).underlined()),
                Span::raw(" at the top left."),
            ]),
            Line::raw(""),
        ];
        // Each entry is a link that opens it. DIR_MARKER stays outside the
        // link because `child_link` needs text that lives for the whole program.
        for (i, child) in entry.children.iter().enumerate() {
            let marker = if child.is_dir() { DIR_MARKER } else { "" };
            lines.push(Line::from(vec![
                Span::raw("  "),
                child_link(child.title, i),
                Span::raw(marker),
            ]));
        }
        return Text::from(lines);
    }
    // A page's text comes from its file in src/content/.
    (entry.text)()
}

/// Where the text goes inside the content pane (inside its border). `draw`
/// and `max_scroll` both use this, so they always wrap to the same width.
/// If you add padding to the pane later, change it here.
fn content_inner(content: Rect) -> Rect {
    content.inner(Margin::new(1, 1))
}

/// The content pane's text, wrapped to `width` columns. `wrap.rs` does the
/// wrapping instead of `Paragraph::wrap`, so bullets get a hanging indent.
fn content_text(entry: &Entry, width: u16) -> Text<'static> {
    crate::wrap::wrap(page_text(entry), width)
}

/// How far the content pane can scroll: the lines (after wrapping) that
/// don't fit in the pane. 0 means everything fits, so it can't scroll at all.
pub fn max_scroll(entry: &Entry, areas: &Areas) -> u16 {
    let inner = content_inner(areas.content);
    let lines = content_text(entry, inner.width).lines.len();
    lines
        .saturating_sub(inner.height as usize)
        .try_into()
        .unwrap_or(u16::MAX)
}

/// How a clickable thing looks while the mouse is over it.
const HOVER_STYLE: Style = Style::new().fg(Color::Black).bg(Color::LightCyan);

/// Draws the home screen from the current `App` state, inside `screen` (the
/// part of the frame the page can show, see `event::visible_grid`). Returns
/// what the mouse is over, if clicking it does something.
pub fn draw(frame: &mut Frame, screen: Rect, app: &App, elapsed: Duration) -> Option<Hovered> {
    let areas = layout(screen, app);
    let Areas {
        whole: area,
        title: title_area,
        menu: left_area,
        content: right_area,
        status: status_area,
    } = areas;

    // rain first so the home screen sits on top of it.
    if app.no_rain {
        crate::rain::view(frame, screen, elapsed);
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

    // Right pane: content for the selected item, already wrapped, scrolled
    let width = content_inner(right_area).width;
    let content = Paragraph::new(content_text(app.current(), width))
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

    // Whatever clickable thing the mouse is over gets highlighted. This runs
    // after the menu and content are drawn because it reads them back from
    // the screen (to find links), the same way a click does.
    let hovered = app
        .hover
        .and_then(|(col, row)| update::hovered(app, frame.buffer_mut(), col, row, &areas));
    if let Some(h) = &hovered {
        for x in h.cols.clone() {
            if let Some(cell) = frame.buffer_mut().cell_mut((x, h.row)) {
                cell.set_style(HOVER_STYLE);
            }
        }
    }

    // Status bar: the keys that do something right now. On the right of its
    // border, like vim's showcmd, the count or "g" being typed.
    let typed = format!(
        "{}{}",
        app.count.map(|c| c.to_string()).unwrap_or_default(),
        app.pending.map(String::from).unwrap_or_default()
    );
    frame.render_widget(
        Paragraph::new(key_hints(app, hovered.as_ref()))
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title("Hints")
                    .title_top(Line::raw(typed).right_aligned())
                    .merge_borders(MergeStrategy::Exact),
            ),
        status_area,
    );

    // Last, so it recolors everything drawn above.
    crate::theme::apply(frame.buffer_mut(), screen, app.theme);
    hovered
}

/// Key binds for the status bar, depending on which pane has focus, whether
/// the highlighted entry is a directory, whether we're inside one, and
/// whether the content can scroll. The line below says what clicking does
/// when the mouse is over something clickable, and has a note otherwise.
/// Keep in sync with `update::update`.
fn key_hints(app: &App, hovered: Option<&Hovered>) -> Text<'static> {
    // A `Text` is a list of lines, so the note always starts on the next line.
    let note = match hovered {
        Some(h) => click_hint(app, h.what),
        None => Line::from(vec![
            Span::styled("Note:", Style::new().fg(Color::LightCyan).bold()),
            Span::raw(" Mouse actions also work!"),
        ]),
    };
    Text::from(vec![key_line(app), note])
}

/// How many rows the key hints take once wrapped in a status bar `width`
/// cells wide, so `layout` can make the status bar tall enough.
fn key_hint_rows(app: &App, width: u16) -> u16 {
    let inside = width.saturating_sub(2); // inside the borders
    let rows = Paragraph::new(key_line(app))
        .wrap(Wrap { trim: true })
        .line_count(inside);
    rows.clamp(1, 3) as u16
}

/// The key hints on one line: each key in the accent color, then what it does.
fn key_line(app: &App) -> Line<'static> {
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

    hints.push(("x", if app.no_rain { "rain off" } else { "rain on" }));
    hints.push((
        "i",
        match app.theme {
            Theme::Dark => "light theme",
            Theme::Light => "dark theme",
        },
    ));

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
    Line::from(spans)
}

/// What clicking `what` does, styled like the key hints: the mouse action,
/// then what it does. Says whether it takes one click or a double-click.
/// Keep in sync with `main.rs` and `update::click` / `update::double_click`.
fn click_hint(app: &App, what: Clickable) -> Line<'static> {
    let action = |s: &'static str| Span::styled(s, Style::new().fg(Color::LightCyan).bold());
    let click = |text: String| Line::from(vec![action("Click"), Span::raw(format!(" {text}"))]);
    match what {
        Clickable::Link(Target::Url(url)) => match url.strip_prefix("mailto:") {
            Some(address) => click(format!("to email {address}")),
            None => click(format!(
                "to open {} in a new tab",
                url.trim_start_matches("https://")
            )),
        },
        Clickable::Link(Target::Child(index)) => {
            let title = app.current().children.get(index).map_or("", |e| e.title);
            click(format!("to open {title}"))
        }
        Clickable::MenuEntry(index) => {
            let entry = &app.menu()[index];
            let name = label(entry);
            if !entry.is_dir() {
                click(format!("to show {name}"))
            } else if index == app.selected {
                // Already previewed, so only a double-click does anything.
                Line::from(vec![action("Double-click"), Span::raw(format!(" to open {name}"))])
            } else {
                Line::from(vec![
                    action("Click"),
                    Span::raw(format!(" to preview {name}   ")),
                    action("Double-click"),
                    Span::raw(" to open it"),
                ])
            }
        }
        Clickable::Breadcrumb(depth) => {
            let parts = app.breadcrumb();
            let name = parts.get(depth).map_or(HOME_LABEL, |p| p.trim_start_matches('/'));
            click(format!("to go back to {name}"))
        }
    }
}
