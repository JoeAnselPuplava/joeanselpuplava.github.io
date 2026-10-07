//! Entry point. Sets up the browser terminal and runs the draw loop.

use ratzilla::ratatui::{Terminal, layout::Rect};

use std::io;

use ratzilla::{DomBackend, WebRenderer};

use crate::content::Target;
use crate::event::EventHandler;
use web_time::Instant;

pub mod app; // state
pub mod content; // the pages: one file per page, one folder per directory
pub mod event; // input
pub mod home; // main layout
pub mod rain; // background animation
pub mod theme; // light and dark themes
pub mod update; // key and click handling
pub mod wrap; // word wrapping with hanging indents for the content pane

fn main() -> io::Result<()> {
    // If the Rust code ever panics, print the message and the file and line
    // in the browser console. Without this a panic only shows up as
    // "unreachable", with no hint of where it happened.
    console_error_panic_hook::set_once();

    // DomBackend draws with HTML elements. To try GPU rendering instead,
    // import WebGl2Backend and use: let backend = WebGl2Backend::new()?;
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let mut app = app::App::new();
    let mut events = EventHandler::new(20)?;
    let start_time = Instant::now();
    let mut pointer_cursor = false; // whether the hand cursor is showing
    let mut page_theme = theme::Theme::default(); // theme of the page around the grid

    // Runs once per browser animation frame (~60 times a second).
    terminal.draw_web(move |f| {
        // Only draw on cells the page actually has (see event::visible_grid).
        // On the very first frame ratzilla hasn't added its grid to the page
        // yet, so draw nothing; the grid is there from the next frame on.
        let Some((cols, rows)) = event::visible_grid() else {
            return;
        };
        let screen = f.area().intersection(Rect::new(0, 0, cols, rows));
        let areas = home::layout(screen, &app);
        app.set_max_scroll(home::max_scroll(app.current(), &areas));

        // 1. Draw the current state. Rain is drawn inside home::draw. It
        // returns what the mouse is over, if that can be clicked: show the
        // hand cursor then. Only touch the page when that changes.
        let hovered = home::draw(f, screen, &app, start_time.elapsed());
        if hovered.is_some() != pointer_cursor {
            pointer_cursor = hovered.is_some();
            event::set_pointer_cursor(pointer_cursor);
        }
        // The grid's colors come from home::draw; the page around it is
        // colored here, and only when the theme changes.
        if app.theme != page_theme {
            page_theme = app.theme;
            event::set_page_background(page_theme.page_background());
        }

        // 2. Apply any input that arrived since the last frame. It's handled
        // after drawing so clicks can be checked against exactly what is on
        // screen (to find links); the changes show up on the next frame.
        for e in events.drain() {
            match e {
                event::Event::Key(key) => update::update(&mut app, key),
                event::Event::Click { col, row } => {
                    match content::link_at(f.buffer_mut(), col, row) {
                        Some(Target::Url(url)) => content::open_link(url),
                        Some(Target::Child(index)) => app.open_child(index),
                        None => update::click(&mut app, col, row, &areas),
                    }
                }
                event::Event::DoubleClick { col, row } => {
                    update::double_click(&mut app, col, row, &areas)
                }
                event::Event::Scroll { col, row, lines } => {
                    update::scroll(&mut app, col, row, lines, &areas)
                }
                event::Event::Hover(cell) => app.hover = cell,
                event::Event::Tick => app.tick(),
            }
        }
    });

    Ok(())
}
