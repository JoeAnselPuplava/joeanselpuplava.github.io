//! Input handling: turns key presses (vim-style motions) and clicks into
//! changes to `App`.

use crate::app::{App, MENU, Pane};
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
        (_, KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter) => app.focus = Pane::Content,
        _ => {} // Esc and unknown keys do nothing; count and pending were already cleared
    }
}

/// Handles a left click on cell (`col`, `row`): clicking a pane focuses it,
/// and clicking a menu entry selects it.
pub fn click(app: &mut App, col: u16, row: u16, areas: &Areas) {
    // A click cancels a half-typed count or "g", like any other key would.
    app.count = None;
    app.pending = None;

    let pos = Position::new(col, row);
    if areas.menu.contains(pos) {
        app.focus = Pane::Menu;
        // Row 0 of the menu area is its top border, so items start at y + 1.
        // Assumes the list never scrolls, which holds while MENU fits on screen.
        if let Some(index) = row.checked_sub(areas.menu.y + 1).map(usize::from)
            && index < MENU.len()
        {
            app.select(index);
        }
    } else if areas.content.contains(pos) {
        app.focus = Pane::Content;
    }
}

/// Handles the mouse wheel at cell (`col`, `row`): over the content it scrolls
/// the text, over the menu it moves the selection. Focus doesn't change.
pub fn scroll(app: &mut App, col: u16, row: u16, lines: i32, areas: &Areas) {
    let pos = Position::new(col, row);
    if areas.content.contains(pos) {
        app.scroll_by(lines);
    } else if areas.menu.contains(pos) {
        let index = (app.selected as i32 + lines).clamp(0, MENU.len() as i32 - 1);
        app.select(index as usize);
    }
}
