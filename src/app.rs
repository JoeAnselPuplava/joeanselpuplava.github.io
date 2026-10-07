//! Application state: everything the UI needs to know to draw a frame.

use ratzilla::ratatui::text::Text;
use std::collections::HashMap;

use crate::theme::Theme;

// The menu itself is defined in `src/content/`, next to the pages.
pub use crate::content::MENU;

/// One item in a menu: either a page, or a directory holding a sub-menu.
#[derive(Debug)]
pub struct Entry {
    pub title: &'static str,
    /// Sub-menu entries. Empty for a normal page.
    pub children: &'static [Entry],
    /// Builds the page's text, e.g. `about::text` from `src/content/about.rs`.
    /// Unused for directories, which list their contents instead.
    pub text: fn() -> Text<'static>,
}

impl Entry {
    /// A normal page whose text comes from `text`.
    pub const fn page(title: &'static str, text: fn() -> Text<'static>) -> Self {
        Self {
            title,
            children: &[],
            text,
        }
    }

    /// A directory: Space opens its sub-menu, Backspace comes back.
    pub const fn dir(title: &'static str, children: &'static [Entry]) -> Self {
        Self {
            title,
            children,
            text: Text::default,
        }
    }

    /// A directory with no children counts as a page, since there is nothing to open.
    pub fn is_dir(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Name of the top-level menu in the breadcrumb. Click it to go back there.
pub const HOME_LABEL: &str = "Home";

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
    pub focus: Pane,                            // pane that j/k act on
    pub path: Vec<usize>, // directories opened, as indices from MENU down (empty = Home)
    pub selected: usize,  // highlighted item in the current menu
    pub scroll: u16,      // scroll offset of the page being shown
    pub saved_scroll: HashMap<Vec<usize>, u16>, // where each page was left, keyed by `key()`
    pub max_scroll: u16,  // furthest `scroll` can go; set every frame in main.rs
    pub count: Option<usize>, // count typed before a motion, e.g. the 5 in "5j"
    pub pending: Option<char>, // first key of a two-key motion, e.g. the first g of "gg"
    pub no_rain: bool,    // Says whether the rain should be turned on or not
    pub hover: Option<(u16, u16)>, // cell under the mouse pointer, as (col, row)
    pub theme: Theme,     // light or dark, switched with i
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
            Pane::Menu => self.select((self.selected + n).min(self.menu().len() - 1)),
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
            Pane::Menu => self.select(self.menu().len() - 1),
            Pane::Content => self.scroll = self.max_scroll,
        }
    }

    /// The menu currently listed on the left: MENU, or an opened directory's children.
    pub fn menu(&self) -> &'static [Entry] {
        self.path.iter().fold(MENU, |menu, &i| menu[i].children)
    }

    /// The highlighted entry, whose page is shown on the right.
    pub fn current(&self) -> &'static Entry {
        &self.menu()[self.selected]
    }

    /// Where we are, for the menu's title: "Home", then "/Name" for each
    /// opened directory. Part `i` is the menu at depth `i`, so clicking it
    /// can call `go_to_depth(i)`.
    pub fn breadcrumb(&self) -> Vec<String> {
        let mut parts = vec![HOME_LABEL.to_string()];
        let mut menu = MENU;
        for &i in &self.path {
            parts.push(format!("/{}", menu[i].title));
            menu = menu[i].children;
        }
        parts
    }

    /// Goes back up until `depth` directories are open (0 = Home), leaving
    /// the directory we came out of highlighted.
    pub fn go_to_depth(&mut self, depth: usize) {
        while self.path.len() > depth {
            self.leave();
        }
    }

    /// Shows entry `index` of the current menu, returning to wherever that
    /// page was last scrolled to. The current page's position is saved first.
    pub fn select(&mut self, index: usize) {
        if index != self.selected {
            self.save_scroll();
            self.selected = index;
            self.load_scroll();
        }
    }

    /// Space: if the highlighted entry is a directory, list its contents.
    pub fn enter(&mut self) {
        if self.current().is_dir() {
            self.save_scroll();
            self.path.push(self.selected);
            self.selected = 0;
            self.load_scroll();
            self.focus = Pane::Menu;
        }
    }

    /// Clicking a link in a directory's preview: opens the directory and
    /// shows its entry `index`, like Space and then moving down to it. Focus
    /// goes to the content, where the click was, so j/k scroll the new page.
    pub fn open_child(&mut self, index: usize) {
        // Two clicks can arrive in one frame and both hit the same link, so
        // the second one finds the directory already open. Do nothing then.
        if !self.current().is_dir() {
            return;
        }
        self.enter();
        self.select(index.min(self.menu().len() - 1));
        self.focus = Pane::Content;
    }

    /// Backspace: go back to the parent menu, with the directory we left highlighted.
    pub fn leave(&mut self) {
        self.save_scroll(); // before popping, so it's saved under this page's key
        if let Some(dir) = self.path.pop() {
            self.selected = dir;
            self.load_scroll();
            self.focus = Pane::Menu;
        }
    }

    /// Identifies the highlighted page across all menus: the path plus its index.
    fn key(&self) -> Vec<usize> {
        let mut key = self.path.clone();
        key.push(self.selected);
        key
    }

    fn save_scroll(&mut self) {
        self.saved_scroll.insert(self.key(), self.scroll);
    }

    fn load_scroll(&mut self) {
        self.scroll = self.saved_scroll.get(&self.key()).copied().unwrap_or(0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_child_opens_the_directory_and_shows_that_entry() {
        // Highlight the first directory on the home menu ("Projects/").
        let dir = MENU.iter().position(Entry::is_dir).unwrap();
        let last = MENU[dir].children.len() - 1;
        let mut app = App::new();
        app.select(dir);

        app.open_child(last);
        assert_eq!(app.path, [dir]);
        assert_eq!(app.selected, last);
        assert_eq!(app.focus, Pane::Content);

        // A second click on the same link (same frame) changes nothing.
        app.open_child(0);
        assert_eq!(app.path, [dir]);
        assert_eq!(app.selected, last);
    }
}
