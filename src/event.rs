//! Input events. The browser calls our callbacks on key/mouse input; they
//! push into a channel, and the render loop drains it once per frame.

use ratzilla::{
    WebRenderer,
    event::{KeyEvent, MouseEvent},
    web_sys::{self, KeyboardEvent},
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
    Mouse(MouseEvent),
}

/// Collects browser input events so the render loop can handle them in order.
#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    /// Registers key and mouse callbacks on the terminal. `tick_rate` is in ms.
    pub fn new<T: WebRenderer>(terminal: &mut T, tick_rate: u64) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel();

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
        web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| std::io::Error::other("no document"))?
            .add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        // Leak the closure on purpose so the listener stays alive for the page's lifetime.
        on_key.forget();

        let mouse_tx = sender;
        terminal
            .on_mouse_event(move |e| {
                let _ = mouse_tx.send(Event::Mouse(e));
            })
            .map_err(std::io::Error::other)?;

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
