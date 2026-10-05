//! Entry point. Sets up the browser terminal and runs the draw loop.

use ratzilla::ratatui::Terminal;

use std::io;
use web_time::Instant;

use ratzilla::{DomBackend, WebRenderer};

use crate::event::EventHandler;

pub mod app; // state
pub mod event; // input
pub mod home; // main layout
pub mod rain; // background animation
pub mod update; // key and click handling

fn main() -> io::Result<()> {
    // DomBackend draws with HTML elements. To try GPU rendering instead,
    // import WebGl2Backend and use: let backend = WebGl2Backend::new()?;
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;
    let start_time = Instant::now();

    let mut app = app::App::new();
    let mut events = EventHandler::new(20)?;

    // Runs once per browser animation frame (~60 times a second).
    terminal.draw_web(move |f| {
        // 1. Apply any input that arrived since the last frame. Clicks are
        // mapped to cells using this frame's size and layout.
        let size = f.area();
        let areas = home::layout(size);
        for e in events.drain() {
            match e {
                event::Event::Key(key) => update::update(&mut app, key),
                event::Event::Click { x, y } => {
                    let col = (x * size.width as f64) as u16;
                    let row = (y * size.height as f64) as u16;
                    update::click(&mut app, col, row, &areas);
                }
                event::Event::Tick => app.tick(),
            }
        }
        // 2. Draw: rain first so the home screen sits on top of it.
        rain::view(f, start_time.elapsed());
        home::draw(f, &app);
    });

    Ok(())
}
