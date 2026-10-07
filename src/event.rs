//! Input events. The browser calls our callbacks on key/mouse input; they
//! push into a channel, and the render loop drains it once per frame.

use ratzilla::{
    event::KeyEvent,
    web_sys::{self, KeyboardEvent, MouseEvent, WheelEvent},
};
use std::sync::mpsc;
use wasm_bindgen::{JsCast, closure::Closure};
// std::time::Instant panics in the browser; web_time works on wasm.
use web_time::{Duration, Instant};

/// Wheel distance (in pixels) that scrolls one line. Lower = faster scrolling.
const PX_PER_LINE: f64 = 20.0;

/// Everything the app reacts to.
#[derive(Clone, Debug)]
pub enum Event {
    /// Fired every `tick_rate` milliseconds.
    Tick,
    Key(KeyEvent),
    /// Left click. `x`/`y` are fractions (0.0-1.0) across and down the
    /// terminal grid; `main.rs` turns them into a cell using the current size.
    Click {
        x: f64,
        y: f64,
    },
    /// Left double-click, positioned like `Click`. The browser sends two
    /// `Click`s before this one.
    DoubleClick {
        x: f64,
        y: f64,
    },
    /// Mouse wheel over the grid: `lines` to scroll (positive = down), with
    /// the pointer position as fractions like `Click`.
    Scroll {
        x: f64,
        y: f64,
        lines: i32,
    },
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
        let click_kinds: [(&str, fn(f64, f64) -> Event); 2] = [
            ("click", |x, y| Event::Click { x, y }),
            ("dblclick", |x, y| Event::DoubleClick { x, y }),
        ];
        for (event_type, make_event) in click_kinds {
            let click_tx = sender.clone();
            let click_doc = document.clone();
            let on_click = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
                if e.button() != 0 {
                    return; // left button only
                }
                if let Some((x, y)) = grid_fraction(&click_doc, &e) {
                    let _ = click_tx.send(make_event(x, y));
                }
            });
            document
                .add_event_listener_with_callback(event_type, on_click.as_ref().unchecked_ref())
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            on_click.forget();
        }

        // Mouse wheel. Trackpads send many tiny pixel deltas, so add them up
        // and only scroll once they reach a whole line.
        let wheel_tx = sender;
        let wheel_doc = document.clone();
        let mut pending_px = 0.0;
        let on_wheel = Closure::<dyn FnMut(WheelEvent)>::new(move |e: WheelEvent| {
            let Some((x, y)) = grid_fraction(&wheel_doc, &e) else {
                return;
            };
            pending_px += match e.delta_mode() {
                WheelEvent::DOM_DELTA_LINE => e.delta_y() * PX_PER_LINE,
                WheelEvent::DOM_DELTA_PAGE => e.delta_y() * PX_PER_LINE * 10.0,
                _ => e.delta_y(), // DOM_DELTA_PIXEL
            };
            let lines = (pending_px / PX_PER_LINE).trunc();
            if lines != 0.0 {
                pending_px -= lines * PX_PER_LINE;
                let _ = wheel_tx.send(Event::Scroll {
                    x,
                    y,
                    lines: lines as i32,
                });
            }
        });
        document
            .add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        on_wheel.forget();

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

/// Where the pointer is over the terminal grid, as fractions (0.0-1.0) of its
/// width and height. `None` when outside the grid. The grid is looked up each
/// time because DomBackend replaces it on resize.
fn grid_fraction(document: &web_sys::Document, e: &MouseEvent) -> Option<(f64, f64)> {
    let rect = document
        .get_element_by_id("grid")?
        .get_bounding_client_rect();
    let x = (e.client_x() as f64 - rect.left()) / rect.width();
    let y = (e.client_y() as f64 - rect.top()) / rect.height();
    // The range check also rejects NaN from a 0-size rect.
    ((0.0..1.0).contains(&x) && (0.0..1.0).contains(&y)).then_some((x, y))
}
