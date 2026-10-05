//! Input events. The browser calls our callbacks on key/mouse input; they
//! push into a channel, and the render loop drains it once per frame.

use ratzilla::{
    event::KeyEvent,
    web_sys::{self, KeyboardEvent, MouseEvent},
};
use std::sync::mpsc;
use wasm_bindgen::{JsCast, closure::Closure};
// std::time::Instant panics in the browser; web_time works on wasm.
use web_time::{Duration, Instant};

/// Everything the app reacts to.
#[derive(Clone, Debug)]
pub enum Event {
    /// Fired every `tick_rate` milliseconds.
    Tick,
    Key(KeyEvent),
    /// Left click. `x`/`y` are fractions (0.0-1.0) across and down the
    /// terminal grid; `main.rs` turns them into a cell using the current size.
    Click { x: f64, y: f64 },
}

/// Collects browser input events so the render loop can handle them in order.
#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    /// Registers key and click listeners on the page. `tick_rate` is in ms.
    pub fn new(tick_rate: u64) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| std::io::Error::other("no document"))?;

        // Each callback gets its own sender. Errors are ignored because the
        // receiver lives as long as the page does.
        let key_tx = sender.clone();

        // Listen for keys on the whole document instead of using
        // `terminal.on_key_event`: that one only fires while the grid element
        // has focus, and DomBackend replaces the grid on window resize, which
        // silently drops the listener.
        let on_key = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            let _ = key_tx.send(Event::Key(e.into()));
        });
        document
            .add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        // Leak the closure on purpose so the listener stays alive for the page's lifetime.
        on_key.forget();

        // Clicks are also listened for on the whole document, for the same
        // resize reason. The grid is looked up on every click so it is never
        // the stale, replaced element.
        let click_tx = sender;
        let click_doc = document.clone();
        let on_click = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            if e.button() != 0 {
                return; // left button only
            }
            let Some(grid) = click_doc.get_element_by_id("grid") else {
                return;
            };
            let rect = grid.get_bounding_client_rect();
            let x = (e.client_x() as f64 - rect.left()) / rect.width();
            let y = (e.client_y() as f64 - rect.top()) / rect.height();
            // Ignore clicks outside the grid (this also skips NaN from a 0-size rect).
            if (0.0..1.0).contains(&x) && (0.0..1.0).contains(&y) {
                let _ = click_tx.send(Event::Click { x, y });
            }
        });
        document
            .add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        on_click.forget();

        Ok(Self {
            receiver,
            tick_rate: Duration::from_millis(tick_rate),
            last_tick: Instant::now(),
        })
    }

    /// Non-blocking: call once per frame and handle everything returned.
    /// Adds a `Tick` if `tick_rate` has passed since the last one.
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = self.receiver.try_iter().collect();
        if self.last_tick.elapsed() >= self.tick_rate {
            events.push(Event::Tick);
            self.last_tick = Instant::now();
        }
        events
    }
}
