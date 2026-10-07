//! The light and dark themes. `i` switches between them.
//!
//! Everything is drawn in the dark theme's colors. When the light theme is
//! on, `apply` goes over the finished frame and swaps each color for its
//! light-theme partner from the tables below, like a palette swap in an old
//! game. That way pages and widgets don't need to know which theme is on.
//! If you start drawing with a new color, add its light partner here.
//!
//! Every light-theme text color has at least 7:1 contrast against the light
//! background, the strictest level in the WCAG accessibility guidelines.

use crate::content::SECTION_COLOR;
use ratzilla::ratatui::{buffer::Buffer, layout::Rect, style::Color};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    /// The other theme.
    pub fn toggled(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    /// The page's background as a CSS color. It shows behind and around the
    /// grid. The dark one matches the `body` style in index.html.
    pub fn page_background(self) -> &'static str {
        match self {
            Theme::Dark => "#121212",
            Theme::Light => "#fafafa",
        }
    }
}

// The light theme's colors. The ratios are contrast against BACKGROUND.
const BACKGROUND: Color = Color::Rgb(0xfa, 0xfa, 0xfa); // same as page_background
const TEXT: Color = Color::Rgb(0x1a, 0x1a, 0x1a); // 16.7:1
const ACCENT: Color = Color::Rgb(0x00, 0x59, 0x6a); // dark teal, 7.6:1
const SECTION: Color = Color::Rgb(0x7c, 0x4a, 0x00); // dark amber, 7.1:1
const RAIN: Color = Color::Rgb(0x7f, 0xb8, 0xb8); // faint on purpose, it's decoration

/// Text colors as (dark theme, light theme).
const LIGHT_FG: &[(Color, Color)] = &[
    (Color::Reset, TEXT),       // normal text and borders (white in the dark theme)
    (Color::White, TEXT),       // the bright heads of the rain drops
    (Color::Black, BACKGROUND), // text on a highlight
    (Color::LightCyan, ACCENT), // headings, links, the focused border, key hints
    (SECTION_COLOR, SECTION),   // section headings
    (Color::Cyan, RAIN),        // the rain
];

/// Background colors as (dark theme, light theme). Cells without a
/// background stay without one, so the page background shows through.
const LIGHT_BG: &[(Color, Color)] = &[
    (Color::White, TEXT), // the selected menu entry: a dark bar instead of a white one
    (Color::LightCyan, ACCENT), // the hover highlight
];

/// Swaps the colors inside `area` (the visible screen) for the light
/// theme's. Does nothing in the dark theme. Call it last, after everything
/// is drawn. It must stay inside `area`: recoloring a blank cell makes it
/// count as drawn, and ratzilla crashes on drawn cells the page doesn't have
/// (see `event::visible_grid`).
pub fn apply(buf: &mut Buffer, area: Rect, theme: Theme) {
    if theme == Theme::Dark {
        return;
    }
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.fg = swap(cell.fg, LIGHT_FG);
            cell.bg = swap(cell.bg, LIGHT_BG);
        }
    }
}

/// `color`'s partner in `table`, or `color` itself if it has none.
fn swap(color: Color, table: &[(Color, Color)]) -> Color {
    table
        .iter()
        .find(|(dark, _)| *dark == color)
        .map_or(color, |(_, light)| *light)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratzilla::ratatui::layout::Rect;

    #[test]
    fn light_theme_swaps_colors_and_dark_leaves_them() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 1));
        buf[(0, 0)].set_fg(Color::LightCyan);
        buf[(1, 0)].set_fg(Color::Black).set_bg(Color::LightCyan);

        let mut dark = buf.clone();
        apply(&mut dark, buf.area, Theme::Dark);
        assert_eq!(dark, buf);

        let area = buf.area; // copied out: `buf` can't be read while lent as `&mut buf`
        apply(&mut buf, area, Theme::Light);
        assert_eq!(buf[(0, 0)].fg, ACCENT);
        assert_eq!(buf[(0, 0)].bg, Color::Reset); // no background stays none
        assert_eq!((buf[(1, 0)].fg, buf[(1, 0)].bg), (BACKGROUND, ACCENT));
    }

    #[test]
    fn light_theme_leaves_cells_outside_the_area_alone() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 2));
        apply(&mut buf, Rect::new(0, 0, 2, 1), Theme::Light);
        assert_eq!(buf[(1, 0)].fg, TEXT); // inside: recolored
        assert_eq!(buf[(2, 0)].fg, Color::Reset); // outside: still blank
        assert_eq!(buf[(0, 1)].fg, Color::Reset);
    }

    /// Contrast ratio between two colors, from the WCAG definition.
    fn contrast(a: Color, b: Color) -> f64 {
        let luminance = |c: Color| {
            let Color::Rgb(r, g, b) = c else {
                panic!("{c:?} isn't an RGB color")
            };
            let channel = |v: u8| {
                let v = f64::from(v) / 255.0;
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
        };
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// Guards readability if the colors above are changed later.
    #[test]
    fn light_text_colors_have_at_least_7_to_1_contrast() {
        for color in [TEXT, ACCENT, SECTION] {
            let ratio = contrast(color, BACKGROUND);
            assert!(ratio >= 7.0, "{color:?} only has {ratio:.1}:1 contrast");
        }
    }
}
