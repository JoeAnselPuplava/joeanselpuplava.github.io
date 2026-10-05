//! Application state: everything the UI needs to know to draw a frame.

/// Entries shown in the left-hand menu. Add a string here to add a page.
pub const MENU: &[&str] = &["About", "Projects", "Experience", "Contact"];

/// Which pane currently receives movement keys (j/k/gg/G).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    #[default]
    Menu,
    Content,
}

/// The whole state of the site. `update.rs` changes it, `home.rs` draws it.
#[derive(Debug, Default)]
pub struct App {
    pub focus: Pane,           // pane that j/k act on
    pub selected: usize,       // highlighted menu item (index into MENU)
    pub scroll: u16,           // scroll offset of the content pane
    pub count: Option<usize>,  // count typed before a motion, e.g. the 5 in "5j"
    pub pending: Option<char>, // first key of a two-key motion, e.g. the first g of "gg"
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called every tick (see `EventHandler`); put timed updates here.
    pub fn tick(&self) {}

    /// Move down `n` lines: next menu item, or scroll the content.
    pub fn move_down(&mut self, n: usize) {
        match self.focus {
            Pane::Menu => self.selected = (self.selected + n).min(MENU.len() - 1),
            Pane::Content => self.scroll = self.scroll.saturating_add(n as u16),
        }
    }

    /// Move up `n` lines. `saturating_sub` stops at 0 instead of underflowing.
    pub fn move_up(&mut self, n: usize) {
        match self.focus {
            Pane::Menu => self.selected = self.selected.saturating_sub(n),
            Pane::Content => self.scroll = self.scroll.saturating_sub(n as u16),
        }
    }

    /// `gg`: jump to the top of the focused pane.
    pub fn top(&mut self) {
        match self.focus {
            Pane::Menu => self.selected = 0,
            Pane::Content => self.scroll = 0,
        }
    }

    /// `G`: jump to the bottom of the focused pane.
    pub fn bottom(&mut self) {
        match self.focus {
            Pane::Menu => self.selected = MENU.len() - 1,
            Pane::Content => self.scroll = u16::MAX, // limit this in draw once you know the text height
        }
    }
}
