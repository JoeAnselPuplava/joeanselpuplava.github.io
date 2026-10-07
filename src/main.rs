//! Entry point. Sets up the browser terminal and runs the draw loop.

use ratzilla::ratatui::Terminal;

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
pub mod update; // key and click handling
pub mod wrap; // word wrapping with hanging indents for the content pane

fn main() -> io::Result<()> {
    // DomBackend draws with HTML elements. To try GPU rendering instead,
    // import WebGl2Backend and use: let backend = WebGl2Backend::new()?;
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let mut app = app::App::new();
    let mut events = EventHandler::new(20)?;
    let start_time = Instant::now();

    // Runs once per browser animation frame (~60 times a second).
    terminal.draw_web(move |f| {
        let size = f.area();
        let areas = home::layout(size);
        app.set_max_scroll(home::max_scroll(app.current(), &areas));

        // 1. Draw the current state. Rain is drawn inside home::draw.
        home::draw(f, &app, start_time.elapsed());

        // 2. Apply any input that arrived since the last frame. It's handled
        // after drawing so clicks can be checked against exactly what is on
        // screen (to find links); the changes show up on the next frame.
        // Clicks are mapped to cells using this frame's size and layout.
        for e in events.drain() {
            match e {
                event::Event::Key(key) => update::update(&mut app, key),
                event::Event::Click { x, y } => {
                    let col = (x * size.width as f64) as u16;
                    let row = (y * size.height as f64) as u16;
                    match content::link_at(f.buffer_mut(), col, row) {
                        Some(Target::Url(url)) => content::open_link(url),
                        Some(Target::Child(index)) => app.open_child(index),
                        None => update::click(&mut app, col, row, &areas),
                    }
                }
                event::Event::DoubleClick { x, y } => {
                    let col = (x * size.width as f64) as u16;
                    let row = (y * size.height as f64) as u16;
                    update::double_click(&mut app, col, row, &areas);
                }
                event::Event::Scroll { x, y, lines } => {
                    let col = (x * size.width as f64) as u16;
                    let row = (y * size.height as f64) as u16;
                    update::scroll(&mut app, col, row, lines, &areas);
                }
                event::Event::Tick => app.tick(),
            }
        }
    });

    Ok(())
}
