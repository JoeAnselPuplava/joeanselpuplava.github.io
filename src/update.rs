//! Key bindings: turns key presses into changes to `App` (vim-style motions).

use crate::app::{App, Pane};
use ratzilla::event::{KeyCode, KeyEvent};

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
