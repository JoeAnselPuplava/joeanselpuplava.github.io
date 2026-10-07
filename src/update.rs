//! Input handling: turns key presses (vim-style motions) and clicks into
//! changes to `App`.

use crate::app::{App, Pane};
use crate::content::{Target, link_extent};
use crate::home::Areas;
use ratzilla::{
    event::{KeyCode, KeyEvent},
    ratatui::{buffer::Buffer, layout::Position},
};
use std::ops::RangeInclusive;

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
        (_, KeyCode::Char('x')) => app.no_rain = !app.no_rain,
        (_, KeyCode::Char('i')) => app.theme = app.theme.toggled(), // light/dark theme
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
/// (0 = Home).
fn breadcrumb_depth(app: &App, col: u16, row: u16, areas: &Areas) -> Option<usize> {
    breadcrumb_part(app, col, row, areas).map(|(depth, _)| depth)
}

/// The breadcrumb part at cell (`col`, `row`): its menu depth and the
/// columns it covers. The title sits on the menu's top border, starting one
/// cell in from the corner.
fn breadcrumb_part(
    app: &App,
    col: u16,
    row: u16,
    areas: &Areas,
) -> Option<(usize, RangeInclusive<u16>)> {
    if row != areas.menu.y {
        return None;
    }
    let mut start = areas.menu.x + 1;
    for (depth, part) in app.breadcrumb().iter().enumerate() {
        let end = start + part.chars().count() as u16;
        if (start..end).contains(&col) {
            return Some((depth, start..=end - 1));
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

/// Something on screen that does something when clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clickable {
    /// A link in the content pane.
    Link(Target),
    /// Entry `index` of the current menu.
    MenuEntry(usize),
    /// The breadcrumb part that goes back to menu depth `depth`.
    Breadcrumb(usize),
}

/// The clickable thing under the pointer and the cells it covers (columns
/// `cols` of row `row`), for the hover highlight and hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hovered {
    pub what: Clickable,
    pub row: u16,
    pub cols: RangeInclusive<u16>,
}

/// What a click at cell (`col`, `row`) would act on, if anything. Checks the
/// same things as a click (links first, like `main.rs`, then the menu) so the
/// hover hint always matches what clicking does.
pub fn hovered(app: &App, buf: &Buffer, col: u16, row: u16, areas: &Areas) -> Option<Hovered> {
    let at = |what, cols| Some(Hovered { what, row, cols });
    if let Some((target, cols)) = link_extent(buf, col, row) {
        return at(Clickable::Link(target), cols);
    }
    if let Some((depth, cols)) = breadcrumb_part(app, col, row, areas) {
        // The last part is where we already are, so clicking it does nothing.
        return if depth < app.path.len() {
            at(Clickable::Breadcrumb(depth), cols)
        } else {
            None
        };
    }
    let index = menu_index(app, col, row, areas)?;
    // The entry's text: after the border and the 2-cell "> " marker column,
    // up to its last character before the right border.
    let start = areas.menu.x + 3;
    let end = (start..areas.menu.right() - 1)
        .rev()
        .find(|&x| buf[(x, row)].symbol() != " ")?;
    at(Clickable::MenuEntry(index), start..=end)
}
