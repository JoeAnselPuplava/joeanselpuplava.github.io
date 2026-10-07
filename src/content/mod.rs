//! The site's content. Every page is a Rust file with a `text()` function, and
//! every directory is a folder with a `mod.rs` listing the pages inside it.
//!
//! To add a page:
//!   1. Create `src/content/<name>.rs` (copy `about.rs` as a starting point).
//!   2. Add `mod <name>;` below.
//!   3. Add `Entry::page("Title", <name>::text)` to `MENU`.
//!
//! To add a directory, do the same with a folder: see `projects/mod.rs`.
//!
//! Helpers for writing pages, so every page looks the same:
//!   - `heading("Title")`: the page title
//!   - `meta("Role | Dates")`: italic details under the title
//!   - `section("Design")`: a heading inside the page
//!   - `bullet("...")` and `labeled_bullet("Label:", "...")`: bullet points
//!   - `link("text", "url")`: a clickable link, used inside a line

use crate::app::Entry;
use ratzilla::{
    ratatui::{
        buffer::Buffer,
        style::{Color, Modifier, Style},
        text::{Line, Span},
    },
    web_sys,
};
use std::{cell::RefCell, collections::HashMap, ops::RangeInclusive};

mod about;
mod contact;
mod experience;
mod projects;

/// The top-level ("Home") menu, in the order it is listed.
pub const MENU: &[Entry] = &[
    Entry::page("About", about::text),
    Entry::dir("Projects", projects::ENTRIES),
    Entry::page("Experience", experience::text),
    Entry::page("Contact", contact::text),
];

/// A page heading in the site's accent color. Use it from any page with
/// `use crate::content::heading;`.
pub fn heading(title: &'static str) -> Line<'static> {
    Line::styled(title, Style::new().fg(Color::LightCyan).bold())
}

/// Color of section headings. Kept different from `heading` and links so
/// the three don't compete for attention.
pub const SECTION_COLOR: Color = Color::Rgb(229, 192, 123);

/// An italic line for the details under a page heading: role, dates, where
/// the source is.
pub fn meta(text: &'static str) -> Line<'static> {
    Line::styled(text, Style::new().italic())
}

/// What every bullet starts with. Wrapped lines of a bullet are indented to
/// line up after it (see `wrap.rs`, which also lists the markers it knows).
pub const BULLET: &str = "  - ";

/// A section heading inside a page, e.g. `section("Design")`. Put a
/// `Line::raw("")` before it to separate it from the section above.
pub fn section(title: &'static str) -> Line<'static> {
    Line::styled(title, Style::new().fg(SECTION_COLOR).bold())
}

/// A bullet point. Takes anything a `Line` can be made from, so both of
/// these work:
///
/// - `bullet("Some text")`
/// - `bullet(vec![Span::raw("See "), link("my repo", "https://...")])`
pub fn bullet(content: impl Into<Line<'static>>) -> Line<'static> {
    let mut line = content.into();
    line.spans.insert(0, Span::raw(BULLET));
    line
}

/// A bullet that starts with a bold label, so a reader can skim the labels
/// and stop at the ones they care about:
/// `labeled_bullet("File encryption:", "Every file is encrypted ...")`.
pub fn labeled_bullet(label: &'static str, text: &'static str) -> Line<'static> {
    bullet(vec![
        Span::styled(label, Style::new().bold()),
        Span::raw(" "),
        Span::raw(text),
    ])
}

/// Invisible style flag that marks a cell as part of a link. SLOW_BLINK is
/// used because the DOM backend doesn't draw it, so it changes nothing visually.
const LINK_MARKER: Modifier = Modifier::SLOW_BLINK;

/// Where a link goes when it's clicked. `main.rs` matches on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A web page or `mailto:` address, opened with `open_link`.
    Url(&'static str),
    /// Entry `index` of the directory whose preview is showing. Clicking it
    /// opens the directory and shows that entry (`App::open_child`).
    Child(usize),
}

thread_local! {
    /// Link text -> where it goes, filled in by `link()` and `child_link()`
    /// as pages are drawn.
    static LINKS: RefCell<HashMap<&'static str, Target>> = RefCell::new(HashMap::new());
}

/// A clickable link showing `text` that opens `url`. Use it inside a line:
/// `Line::from(vec![Span::raw("GitHub: "), link("github.com/me", "https://github.com/me")])`.
/// Use `mailto:you@example.com` as the URL for an email address.
///
/// Links are found by their text, so give each link on the site its own text.
pub fn link(text: &'static str, url: &'static str) -> Span<'static> {
    link_span(text, Target::Url(url))
}

/// A link to entry `index` of the directory being previewed, e.g. one of the
/// projects listed when "Projects/" is highlighted. Used by `home::page_text`.
pub fn child_link(text: &'static str, index: usize) -> Span<'static> {
    link_span(text, Target::Child(index))
}

/// Remembers where `text` leads and styles it as a link.
fn link_span(text: &'static str, target: Target) -> Span<'static> {
    LINKS.with_borrow_mut(|links| links.insert(text, target));
    Span::styled(
        text,
        Style::new().fg(Color::LightCyan).underlined().add_modifier(LINK_MARKER),
    )
}

/// Where the link drawn at cell (`col`, `row`) of `buf` goes, if there is
/// one. Reads the link's text back from the screen, so it works wherever
/// wrapping and scrolling put it.
pub fn link_at(buf: &Buffer, col: u16, row: u16) -> Option<Target> {
    link_extent(buf, col, row).map(|(target, _)| target)
}

/// Like `link_at`, but also returns the columns the link covers on `row`,
/// for the hover highlight.
pub fn link_extent(buf: &Buffer, col: u16, row: u16) -> Option<(Target, RangeInclusive<u16>)> {
    let is_link = |x: u16| {
        buf.cell((x, row))
            .is_some_and(|cell| cell.modifier.contains(LINK_MARKER))
    };
    if !is_link(col) {
        return None;
    }
    // Spread out from the clicked cell to the whole link on this row.
    let mut start = col;
    while start > 0 && is_link(start - 1) {
        start -= 1;
    }
    let mut end = col;
    while is_link(end + 1) {
        end += 1;
    }
    let text: String = (start..=end).map(|x| buf[(x, row)].symbol()).collect();
    let text = text.trim();

    let target = LINKS.with_borrow(|links| {
        links.get(text).copied().or_else(|| {
            // A link wrapped onto two lines only shows part of its text on
            // each, so fall back to the link that contains this part.
            let mut matches = links.iter().filter(|(full, _)| full.contains(text));
            match (matches.next(), matches.next()) {
                (Some((_, target)), None) => Some(*target),
                _ => None,
            }
        })
    })?;
    Some((target, start..=end))
}

/// Opens `url`: email links in the visitor's mail app, other links in a new tab.
pub fn open_link(url: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if url.starts_with("mailto:") {
        let _ = window.location().set_href(url);
    } else if !matches!(window.open_with_url_and_target(url, "_blank"), Ok(Some(_))) {
        // The browser blocked the new tab (popup blocker), so open it here instead.
        let _ = window.location().set_href(url);
    }
}
