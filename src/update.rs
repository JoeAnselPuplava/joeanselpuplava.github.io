//! Input handling: turns key presses (vim-style motions) and clicks into
//! changes to `App`.

use crate::app::{App, Pane};
use crate::event::Event::Key;
use crate::home::Areas;
use ratzilla::{
    event::{KeyCode, KeyEvent},
    ratatui::layout::Position,
};

/// Handles one key press. To add a binding, add an arm to the `match`.
pub fn update(app: &mut App, key: KeyEvent) {
    // Count prefix: 1-9 starts a count, 0 only continues one (so "10j" works)
    if let KeyCode::Char(c @ '0'..='9') = key.code
        && (c != '0' || app.count.is_some())
    {
        app.count = Some(app.count.unwrap_or(0) * 10 + c.to_digit(10).unwrap() as usize);
        return;
    }
    let n = app.count.take().unwrap_or(1);
    let pending = app.pending.take(); // any key other than the one that completes a motion cancels it, like in vim

    // Match on (previous key, this key) so two-key motions like "gg" work.
    match (pending, key.code) {
        (_, KeyCode::Char('d')) if key.ctrl => app.move_down(10), // half-page down
        (_, KeyCode::Char('u')) if key.ctrl => app.move_up(10),   // half-page up
        (Some('g'), KeyCode::Char('g')) => app.top(),
        (None, KeyCode::Char('g')) => app.pending = Some('g'), // wait for the second g
        (_, KeyCode::Char('G')) => app.bottom(),
        (_, KeyCode::Char('j') | KeyCode::Down) => app.move_down(n),
        (_, KeyCode::Char('k') | KeyCode::Up) => app.move_up(n),
        (_, KeyCode::Char('h') | KeyCode::Left) => app.focus = Pane::Menu,
        (_, KeyCode::Char('l') | KeyCode::Right) => app.focus = Pane::Content,
        (_, KeyCode::Char(' ') | KeyCode::Enter) => app.enter(), // open a directory
        (_, KeyCode::Backspace) => app.leave(),                  // back to the parent menu
        _ => {} // Esc and unknown keys do nothing; count and pending were already cleared
    }
}

/// Handles a left click on cell (`col`, `row`): clicking a pane focuses it,
/// clicking a menu entry selects it, and clicking part of the breadcrumb in
/// the menu's title goes back to that menu.
pub fn click(app: &mut App, col: u16, row: u16, areas: &Areas) {
    // A click cancels a half-typed count or "g", like any other key would.
    app.count = None;
    app.pending = None;

    let pos = Position::new(col, row);
    if areas.menu.contains(pos) {
        app.focus = Pane::Menu;
        if let Some(depth) = breadcrumb_depth(app, col, row, areas) {
            app.go_to_depth(depth);
        } else if let Some(index) = menu_index(app, col, row, areas) {
            app.select(index);
        }
    } else if areas.content.contains(pos) {
        app.focus = Pane::Content;
    }
}

/// Handles a left double-click: on a directory in the menu it opens it, like
/// Space. The two single clicks before it have already selected the entry.
pub fn double_click(app: &mut App, col: u16, row: u16, areas: &Areas) {
    // Only when the pointer is on an entry, so a double-click on the empty
    // space below the list doesn't open whatever happens to be highlighted.
    if let Some(index) = menu_index(app, col, row, areas) {
        app.select(index);
        app.enter();
    }
}

/// Which part of the breadcrumb is at cell (`col`, `row`), as a menu depth
/// (0 = Home). The title sits on the menu's top border, starting one cell in
/// from the corner.
fn breadcrumb_depth(app: &App, col: u16, row: u16, areas: &Areas) -> Option<usize> {
    if row != areas.menu.y {
        return None;
    }
    let mut start = areas.menu.x + 1;
    for (depth, part) in app.breadcrumb().iter().enumerate() {
        let end = start + part.chars().count() as u16;
        if (start..end).contains(&col) {
            return Some(depth);
        }
        start = end;
    }
    None
}

/// Which entry of the current menu is at cell (`col`, `row`), if any.
fn menu_index(app: &App, col: u16, row: u16, areas: &Areas) -> Option<usize> {
    if !areas.menu.contains(Position::new(col, row)) {
        return None;
    }
    // Row 0 of the menu area is its top border, so items start at y + 1.
    // Assumes the list never scrolls, which holds while the menu fits on screen.
    let index = usize::from(row.checked_sub(areas.menu.y + 1)?);
    (index < app.menu().len()).then_some(index)
}

/// Handles the mouse wheel at cell (`col`, `row`): over the content it scrolls
/// the text, over the menu it moves the selection. Focus doesn't change.
pub fn scroll(app: &mut App, col: u16, row: u16, lines: i32, areas: &Areas) {
    let pos = Position::new(col, row);
    if areas.content.contains(pos) {
        app.scroll_by(lines);
    } else if areas.menu.contains(pos) {
        let index = (app.selected as i32 + lines).clamp(0, app.menu().len() as i32 - 1);
        app.select(index as usize);
    }
}
