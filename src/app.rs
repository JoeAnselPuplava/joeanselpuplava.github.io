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
    pub scroll: u16,           // scroll offset of the page being shown
    pub saved_scroll: [u16; MENU.len()], // where each page was left, restored by `select`
    pub max_scroll: u16,       // furthest `scroll` can go; set every frame in main.rs
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
            Pane::Menu => self.select((self.selected + n).min(MENU.len() - 1)),
            Pane::Content => self.scroll_by(n as i32),
        }
    }

    /// Move up `n` lines. `saturating_sub` stops at 0 instead of underflowing.
    pub fn move_up(&mut self, n: usize) {
        match self.focus {
            Pane::Menu => self.select(self.selected.saturating_sub(n)),
            Pane::Content => self.scroll_by(-(n as i32)),
        }
    }

    /// `gg`: jump to the top of the focused pane.
    pub fn top(&mut self) {
        match self.focus {
            Pane::Menu => self.select(0),
            Pane::Content => self.scroll = 0,
        }
    }

    /// `G`: jump to the bottom of the focused pane.
    pub fn bottom(&mut self) {
        match self.focus {
            Pane::Menu => self.select(MENU.len() - 1),
            Pane::Content => self.scroll = self.max_scroll,
        }
    }

    /// Shows menu entry `index`, returning to wherever that page was last
    /// scrolled to. The current page's position is saved first.
    pub fn select(&mut self, index: usize) {
        if index != self.selected {
            self.saved_scroll[self.selected] = self.scroll;
            self.selected = index;
            self.scroll = self.saved_scroll[index];
        }
    }

    /// Scrolls the content by `lines` (negative = up), staying within
    /// 0..=max_scroll so it can't scroll past the end of the text.
    pub fn scroll_by(&mut self, lines: i32) {
        self.scroll = (self.scroll as i32 + lines).clamp(0, self.max_scroll as i32) as u16;
    }

    /// Updates how far the content can scroll (it changes with the page and
    /// window size) and pulls `scroll` back if it is now past the end.
    pub fn set_max_scroll(&mut self, max: u16) {
        self.max_scroll = max;
        self.scroll = self.scroll.min(max);
    }
}
