//! Home screen: title, menu, content pane and status bar.

use ratzilla::ratatui::{
    Frame,
    layout::{
        Constraint::{self, Percentage, Ratio},
        Layout, Rect, Spacing,
    },
    style::{Color, Style},
    symbols::merge::MergeStrategy,
    text::Text,
    widgets::{Block, Clear, List, ListState, Paragraph},
};

use crate::app::{App, MENU, Pane};

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

/// Draws the home screen from the current `App` state.
pub fn draw(frame: &mut Frame, app: &App) {
    let Areas {
        whole: area,
        title: title_area,
        menu: left_area,
        content: right_area,
        status: status_area,
    } = layout(frame.area());

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
    let list = List::new(MENU.iter().copied())
        .block(
            Block::bordered()
                .merge_borders(MergeStrategy::Exact)
                .border_style(focused(Pane::Menu)),
        )
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold().black().bg(Color::White));
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, left_area, &mut state);

    // Right pane: content for the selected item, scrolled
    let content = Paragraph::new(format!("Content for {}", MENU[app.selected]))
        .block(
            Block::bordered()
                .merge_borders(MergeStrategy::Exact)
                .border_style(focused(Pane::Content)),
        )
        .scroll((app.scroll, 0));
    frame.render_widget(content, right_area);

    // Status bar: like vim's showcmd, display the count or "g" being typed
    let typed = format!(
        "{}{}",
        app.count.map(|c| c.to_string()).unwrap_or_default(),
        app.pending.map(String::from).unwrap_or_default()
    );
    frame.render_widget(
        Paragraph::new(typed).block(
            Block::bordered()
                .title("Status Bar")
                .merge_borders(MergeStrategy::Exact),
        ),
        status_area,
    );
}
