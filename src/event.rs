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
    /// Left click on the terminal cell at column `col`, row `row`.
    Click { col: u16, row: u16 },
    /// Left double-click on a cell. The browser sends two `Click`s before
    /// this one.
    DoubleClick { col: u16, row: u16 },
    /// Mouse wheel over a cell: `lines` to scroll (positive = down).
    Scroll { col: u16, row: u16, lines: i32 },
    /// The pointer moved: the cell it is over as (col, row), or `None` once
    /// it leaves the grid or the page.
    Hover(Option<(u16, u16)>),
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
        let click_kinds: [(&str, fn(u16, u16) -> Event); 2] = [
            ("click", |col, row| Event::Click { col, row }),
            ("dblclick", |col, row| Event::DoubleClick { col, row }),
        ];
        for (event_type, make_event) in click_kinds {
            let click_tx = sender.clone();
            let click_doc = document.clone();
            let on_click = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
                if e.button() != 0 {
                    return; // left button only
                }
                if let Some((col, row)) = grid_cell(&click_doc, &e) {
                    let _ = click_tx.send(make_event(col, row));
                }
            });
            document
                .add_event_listener_with_callback(event_type, on_click.as_ref().unchecked_ref())
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            on_click.forget();
        }

        // Pointer movement, for the hover highlight. Leaving the page counts
        // as hovering nothing, so a highlight doesn't stay stuck on screen.
        let move_tx = sender.clone();
        let move_doc = document.clone();
        let on_move = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            let _ = move_tx.send(Event::Hover(grid_cell(&move_doc, &e)));
        });
        document
            .add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        on_move.forget();
        let leave_tx = sender.clone();
        let on_leave = Closure::<dyn FnMut(MouseEvent)>::new(move |_: MouseEvent| {
            let _ = leave_tx.send(Event::Hover(None));
        });
        if let Some(root) = document.document_element() {
            root.add_event_listener_with_callback("mouseleave", on_leave.as_ref().unchecked_ref())
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        }
        on_leave.forget();

        // Mouse wheel. Trackpads send many tiny pixel deltas, so add them up
        // and only scroll once they reach a whole line.
        let wheel_tx = sender;
        let wheel_doc = document.clone();
        let mut pending_px = 0.0;
        let on_wheel = Closure::<dyn FnMut(WheelEvent)>::new(move |e: WheelEvent| {
            let Some((col, row)) = grid_cell(&wheel_doc, &e) else {
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
                    col,
                    row,
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

/// The terminal cell under the pointer, as (col, row). `None` when outside
/// the grid. The grid is looked up each time because DomBackend replaces it
/// on resize.
///
/// The cell is counted on the grid as it is drawn in the page (one `<pre>`
/// per row, one `<span>` per cell), not from the size ratatui is told.
/// ratzilla 0.3.1 gives ratatui a rough, smaller size (window width / 10 and
/// height / 20, minus 1), so the page's grid has more rows and columns than
/// ratatui's frame. Ratatui draws into the grid's top-left corner, so the
/// cell under the pointer is the same in both, but scaling by ratatui's size
/// would land clicks rows and columns off, more so toward the bottom right.
fn grid_cell(document: &web_sys::Document, e: &MouseEvent) -> Option<(u16, u16)> {
    let grid = document.get_element_by_id("grid")?;
    let rows = grid.child_element_count();
    let cols = grid.first_element_child()?.child_element_count();
    let rect = grid.get_bounding_client_rect();
    let x = (e.client_x() as f64 - rect.left()) / rect.width();
    let y = (e.client_y() as f64 - rect.top()) / rect.height();
    // The range check also rejects NaN from a 0-size rect.
    if !((0.0..1.0).contains(&x) && (0.0..1.0).contains(&y)) {
        return None;
    }
    Some(((x * cols as f64) as u16, (y * rows as f64) as u16))
}

/// How many (columns, rows) of cells the page can show, so nothing is drawn
/// outside them. `None` until ratzilla has put its grid on the page, which
/// happens at the end of the very first frame.
///
/// ratzilla 0.3.1 tells ratatui a screen size guessed from the window
/// (width / 10 and height / 20 pixels, minus 1), but builds the page's grid
/// from the real character size. When characters are bigger than that guess
/// (a larger font setting in the browser, or Firefox measuring them a little
/// taller than Chrome), the grid has fewer rows or columns than ratatui's
/// screen, and ratzilla crashes when asked to draw a cell the grid doesn't
/// have. (A crash in WebAssembly stops the draw loop, so the page freezes.)
///
/// Besides counting the grid that is there now, this works out the size
/// ratzilla will give the grid after a window resize, the same way ratzilla
/// does (page size / character size), because ratzilla only rebuilds the
/// grid after this frame is drawn.
pub fn visible_grid() -> Option<(u16, u16)> {
    let document = web_sys::window()?.document()?;
    let grid = document.get_element_by_id("grid")?;
    let rows = f64::from(grid.child_element_count());
    let cols = f64::from(grid.first_element_child()?.child_element_count());
    if rows == 0.0 || cols == 0.0 {
        return None;
    }
    let rect = grid.get_bounding_client_rect();
    let (cell_width, cell_height) = (rect.width() / cols, rect.height() / rows);
    let page = document.body()?.get_bounding_client_rect();
    let fit_cols = (page.width() / cell_width).floor();
    let fit_rows = (page.height() / cell_height).floor();
    Some((cols.min(fit_cols) as u16, rows.min(fit_rows) as u16))
}

/// Sets the page's background color: the `<body>` behind and around the
/// grid. Cells without a background of their own let it show through, so
/// this colors most of the screen.
pub fn set_page_background(css_color: &str) {
    let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    else {
        return;
    };
    let _ = body.set_attribute("style", &format!("background-color: {css_color}"));
}

/// Shows the hand cursor while `on`, the browser's usual sign that clicking
/// does something. Sets the style on the `<html>` element, and every cell
/// inherits it.
pub fn set_pointer_cursor(on: bool) {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    else {
        return;
    };
    let _ = if on {
        root.set_attribute("style", "cursor: pointer")
    } else {
        root.remove_attribute("style")
    };
}
