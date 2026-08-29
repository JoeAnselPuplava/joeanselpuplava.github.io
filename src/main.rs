use ratzilla::ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Rect},
    style::Color,
    widgets::{Block, Paragraph},
};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use ratzilla::{DomBackend, WebGl2Backend, WebRenderer, event::KeyCode};

mod home;

fn main() -> io::Result<()> {
    let counter = Rc::new(RefCell::new(0));
    // let backend = WebGl2Backend::new()?;
    let backend = DomBackend::new()?;
    let mut terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let counter_cloned = counter.clone();
        move |key_event| {
            if key_event.code == KeyCode::Char(' ') {
                let mut counter = counter_cloned.borrow_mut();
                *counter += 1;
            }
        }
    })?;

    //This draw doesn't work
    terminal.draw_web(home::draw);

    // This draw does work
    // terminal.draw_web(move |f| {
    //     let counter = counter.borrow();
    //     f.render_widget(
    //         Paragraph::new(format!("Count: {counter}"))
    //             .alignment(Alignment::Center)
    //             .block(
    //                 Block::bordered()
    //                     .title("Joe-Ansel Puplava")
    //                     .title_alignment(Alignment::Center)
    //                     .border_style(Color::Cyan),
    //             ),
    //         Rect::new(
    //             f.area().width / 4,
    //             f.area().height / 4,
    //             f.area().width / 2,
    //             f.area().height / 2,
    //         ),
    //     );
    // });

    Ok(())
}
