//! Word wrapping for the content pane, with hanging indents.
//!
//! ratatui's `Paragraph::wrap` starts every wrapped line at the left edge, so
//! a long bullet's text ends up under its "-". `wrap` does the wrapping itself
//! and lines wrapped text up under the first word instead:
//!
//! ```text
//!   - Every file is encrypted with AES-CBC        - Every file is encrypted with AES-CBC
//!   under a fresh random key.            instead of   under a fresh random key.
//! ```
//!
//! The indent for wrapped lines is the line's leading spaces, plus a bullet
//! marker from `MARKERS` and the space after it. So `content::bullet(...)`,
//! a hand-written `Line::raw("  - ...")` and an indented paragraph all work.

use ratzilla::ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span, Text},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Bullet markers that get a hanging indent when followed by a space.
const MARKERS: &[&str] = &["-", "*", "•"];

/// Wraps every line of `text` to fit in `width` columns.
pub fn wrap(mut text: Text<'static>, width: u16) -> Text<'static> {
    if width == 0 {
        return text; // nothing fits; leave it for ratatui to clip
    }
    text.lines = std::mem::take(&mut text.lines)
        .into_iter()
        .flat_map(|line| wrap_line(line, width.into()))
        .collect();
    text
}

/// Wraps one line. The first line keeps the bullet or indent it started with;
/// the rest are indented to line up with the text after it.
fn wrap_line(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    let prefix_width = hanging_indent(&line);
    let (prefix, rest) = split_prefix(line.spans, prefix_width);
    // On a very narrow pane don't spend more than half the width on indent.
    let indent = if prefix_width * 2 <= width {
        prefix_width
    } else {
        0
    };

    let mut out = Wrapper {
        width,
        indent,
        style: line.style,
        alignment: line.alignment,
        lines: Vec::new(),
        current: prefix,
        used: prefix_width,
        empty: true,
    };

    // The spaces after the previous word. They are only written if another
    // word follows on the same line, so wrapped lines never start or end
    // with spaces.
    let mut gap: Vec<Span<'static>> = Vec::new();
    for word in words(rest) {
        let word_width = width_of(&word.text);
        if !out.empty {
            if out.used + width_of(&gap) + word_width > width {
                out.new_line();
            } else {
                out.push_spans(gap);
            }
        }
        for piece in word.text {
            out.push_text(&piece.content, piece.style);
        }
        gap = word.space;
    }
    out.finish()
}

/// How many columns the wrapped lines of `line` should be indented by.
fn hanging_indent(line: &Line) -> usize {
    let text: String = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    let rest = text.trim_start_matches(' ');
    let spaces = text.len() - rest.len(); // spaces are 1 byte and 1 column each
    let marker = MARKERS.iter().find(|&&marker| {
        rest.strip_prefix(marker)
            .is_some_and(|after| after.starts_with(' '))
    });
    match marker {
        Some(marker) => spaces + marker.width() + 1,
        None => spaces,
    }
}

/// Splits `spans` into the first `columns` columns and everything after.
fn split_prefix(
    spans: Vec<Span<'static>>,
    columns: usize,
) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
    let mut prefix = Vec::new();
    let mut rest = Vec::new();
    let mut left = columns;
    for span in spans {
        if left == 0 {
            rest.push(span);
            continue;
        }
        let (head, tail) = split_at_width(&span.content, left);
        left -= head.width();
        prefix.push(Span::styled(head.to_string(), span.style));
        if !tail.is_empty() {
            rest.push(Span::styled(tail.to_string(), span.style));
        }
    }
    (prefix, rest)
}

/// A word and the spaces after it. Both are kept as styled pieces so a bold
/// label or a link keeps its style when it moves to the next line.
#[derive(Default)]
struct Word {
    text: Vec<Span<'static>>,
    space: Vec<Span<'static>>,
}

/// Splits styled text into words. A word can be made of several spans, e.g.
/// a link followed by a period, and moves to the next line as one piece.
fn words(spans: Vec<Span<'static>>) -> Vec<Word> {
    let mut words = Vec::new();
    let mut word = Word::default();
    for span in spans {
        for (is_space, run) in runs(&span.content) {
            let piece = Span::styled(run.to_string(), span.style);
            if is_space {
                word.space.push(piece);
            } else {
                // Text after a space starts a new word.
                if !word.space.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
                word.text.push(piece);
            }
        }
    }
    if !word.text.is_empty() || !word.space.is_empty() {
        words.push(word);
    }
    words
}

/// Splits `s` into runs of spaces and runs of everything else, in order:
/// "a  bc d" gives (false, "a"), (true, "  "), (false, "bc"), (true, " "), (false, "d").
fn runs(s: &str) -> impl Iterator<Item = (bool, &str)> {
    let mut rest = s;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let is_space = rest.starts_with(' ');
        let end = rest
            .find(|c: char| (c == ' ') != is_space)
            .unwrap_or(rest.len());
        let (run, tail) = rest.split_at(end);
        rest = tail;
        Some((is_space, run))
    })
}

/// Splits `s` after as many characters as fit in `columns`.
fn split_at_width(s: &str, columns: usize) -> (&str, &str) {
    let mut used = 0;
    for (i, c) in s.char_indices() {
        used += c.width().unwrap_or(0);
        if used > columns {
            return s.split_at(i);
        }
    }
    (s, "")
}

/// Total width in columns of some spans.
fn width_of(spans: &[Span]) -> usize {
    spans.iter().map(|span| span.content.width()).sum()
}

/// Collects the wrapped lines of one source line.
struct Wrapper {
    width: usize,
    indent: usize,
    // Copied from the source line onto every wrapped line, so e.g. a whole
    // `Line::raw(...).italic()` stays italic after wrapping.
    style: Style,
    alignment: Option<Alignment>,
    lines: Vec<Line<'static>>,
    current: Vec<Span<'static>>,
    used: usize, // columns used on `current`, indent included
    empty: bool, // no words on `current` yet
}

impl Wrapper {
    /// Ends the current line and starts an indented one.
    fn new_line(&mut self) {
        let spans = std::mem::take(&mut self.current);
        self.lines.push(self.line(spans));
        self.current = vec![Span::raw(" ".repeat(self.indent))];
        self.used = self.indent;
        self.empty = true;
    }

    /// Adds spans that are known to fit.
    fn push_spans(&mut self, spans: Vec<Span<'static>>) {
        self.used += width_of(&spans);
        self.current.extend(spans);
    }

    /// Adds part of a word. A word longer than a whole line is broken
    /// across lines, which only happens for long URLs on narrow screens.
    fn push_text(&mut self, text: &str, style: Style) {
        let mut rest = text;
        while !rest.is_empty() {
            let (mut fit, mut after) = split_at_width(rest, self.width.saturating_sub(self.used));
            if fit.is_empty() {
                if !self.empty {
                    self.new_line();
                    continue;
                }
                // Not even one character fits on an empty line, e.g. a
                // 2-column character on a 1-column pane. Place it anyway so
                // the loop always moves forward.
                let first = rest.chars().next().map_or(0, char::len_utf8);
                (fit, after) = rest.split_at(first);
            }
            self.used += fit.width();
            self.current.push(Span::styled(fit.to_string(), style));
            self.empty = false;
            rest = after;
        }
    }

    /// Ends the last line and returns all of them.
    fn finish(mut self) -> Vec<Line<'static>> {
        let spans = std::mem::take(&mut self.current);
        self.lines.push(self.line(spans));
        self.lines
    }

    fn line(&self, spans: Vec<Span<'static>>) -> Line<'static> {
        Line {
            spans,
            style: self.style,
            alignment: self.alignment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratzilla::ratatui::style::{Modifier, Stylize};

    /// The text of each wrapped line, without styles.
    fn plain(text: &Text) -> Vec<String> {
        text.lines.iter().map(|line| line.to_string()).collect()
    }

    #[test]
    fn bullets_get_a_hanging_indent() {
        let text = Text::from(Line::raw("  - one two three four five"));
        assert_eq!(
            plain(&wrap(text, 14)),
            ["  - one two", "    three four", "    five"]
        );
    }

    #[test]
    fn indented_paragraphs_keep_their_indent() {
        let text = Text::from(Line::raw("  one two three"));
        assert_eq!(plain(&wrap(text, 9)), ["  one two", "  three"]);
    }

    #[test]
    fn plain_paragraphs_wrap_to_the_left_edge() {
        let text = Text::from(Line::raw("I led a team of five"));
        assert_eq!(plain(&wrap(text, 10)), ["I led a", "team of", "five"]);
    }

    #[test]
    fn styles_survive_wrapping() {
        let line = Line::from(vec![
            Span::raw("  - "),
            Span::raw("Label:").bold(),
            Span::raw(" some words here"),
        ]);
        let wrapped = wrap(Text::from(line), 12);
        assert_eq!(
            plain(&wrapped),
            ["  - Label:", "    some", "    words", "    here"]
        );
        let label = &wrapped.lines[0].spans[1];
        assert_eq!(label.content, "Label:");
        assert!(label.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn line_style_is_copied_to_every_wrapped_line() {
        let wrapped = wrap(Text::from(Line::raw("aa bb cc").italic()), 4);
        assert_eq!(wrapped.lines.len(), 3);
        assert!(
            wrapped
                .lines
                .iter()
                .all(|l| l.style.add_modifier.contains(Modifier::ITALIC))
        );
    }

    #[test]
    fn spaces_between_words_are_kept() {
        // Like the Contact page, where spaces line the links up.
        let text = Text::from(Line::raw("Email:    me@example.com"));
        assert_eq!(plain(&wrap(text, 40)), ["Email:    me@example.com"]);
    }

    #[test]
    fn words_longer_than_a_line_are_broken() {
        let text = Text::from(Line::raw("see github.com/abcdefgh"));
        assert_eq!(plain(&wrap(text, 10)), ["see", "github.com", "/abcdefgh"]);
    }

    #[test]
    fn blank_lines_stay() {
        let text = Text::from(vec![Line::raw("a"), Line::raw(""), Line::raw("b")]);
        assert_eq!(plain(&wrap(text, 10)), ["a", "", "b"]);
    }
}
