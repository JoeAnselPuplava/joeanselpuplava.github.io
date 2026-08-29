use ratzilla::ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Rect},
    style::Color,
    widgets::{Block, Paragraph},
};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use web_time::Instant;

use ratzilla::{DomBackend, WebGl2Backend, WebRenderer, event::KeyCode};

mod home;
mod rain;

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    // let backend = WebGl2Backend::new()?;
    let backend = DomBackend::new()?;
    let mut terminal = Terminal::new(backend)?;
    let start_time = Instant::now();

    terminal.on_key_event({
        let counter_cloned = counter.clone();
        move |key_event| {
            if key_event.code == KeyCode::Char(' ') {
                let mut counter = counter_cloned.borrow_mut();
                *counter += 1;
            }
        }
    })?;

    terminal.draw_web(move |f| {
        rain::view(f, start_time.elapsed());
        home::draw(f);
    });

    Ok(())
}
