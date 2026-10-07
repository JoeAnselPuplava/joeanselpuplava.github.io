//! Project: CPS-IR optimizer.

use crate::content::{bullet, heading, meta, section};
use ratzilla::ratatui::text::{Line, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("CPS-IR Optimizations"),
        meta("Compilers (CS 107), Tufts University | Mar. 2026 - Apr. 2026"),
        meta("Source available on request (course project)"),
        Line::raw(""),
        Line::raw(
            "An optimizer for our class's MiniScala compiler, written in Scala. The compiler translates programs into a continuation-passing style (CPS) intermediate representation, where every call and return is explicit. That makes programs easy to analyze, but the direct translation is full of tiny functions and closures that the optimizer has to clean up.",
        ),
        Line::raw(""),
        section("How it works"),
        bullet("Shrinking optimizations, which can only make the program smaller, run over and over until nothing changes: dead-code elimination, common-subexpression elimination, constant folding, and simplifying operations with neutral or absorbing elements (like x + 0 or x * 0)."),
        bullet("Shrinking inlining replaces functions and continuations that are only used once with their bodies."),
        bullet("General inlining then copies larger functions into their call sites. Heuristics limit how much code it can add (a Fibonacci-based size limit for functions and a linear one for continuations), and another round of shrinking follows. This repeats until the program stops changing, hits an iteration limit, or grows too large."),
        Line::raw(""),
        section("Results"),
        bullet("Optimized programs down to minimal straight-line code by eliminating redundant closures and dead branches"),
        bullet("Validated against blackbox tests (programs still produce the same output) and greybox tests (execution traces show the expected optimizations happened)"),
    ])
}
