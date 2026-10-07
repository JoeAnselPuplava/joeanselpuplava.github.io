//! Project: mark-and-sweep garbage collector.

use crate::content::{heading, labeled_bullet, meta, section};
use ratzilla::ratatui::text::{Line, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Mark & Sweep Garbage Collector"),
        meta("Compilers (CS 107), Tufts University | Apr. 2026"),
        Line::raw(""),
        Line::raw(
            "A garbage collector written in C for the virtual machine that runs programs from our class's MiniScala compiler. Before this project the VM never freed memory, so larger programs simply ran out. The collector works inside one fixed block of memory handed to the VM at startup and never calls malloc.",
        ),
        Line::raw(""),
        section("Design"),
        labeled_bullet(
            "Block headers:",
            "Every block starts with a one-word header that packs the block's tag and size together.",
        ),
        Line::raw(""),
        labeled_bullet(
            "Allocation:",
            "Free blocks live in 32 segregated free lists: one for each small size and a final list for everything larger. Allocation starts at the list for the requested size and splits a bigger block when it has to.",
        ),
        Line::raw(""),
        labeled_bullet(
            "Heap bitmap:",
            "A bitmap with one bit per heap word sits in front of the heap. It does two jobs. It records which words are the start of a real block, so integers aren't mistaken for pointers, and it doubles as the mark bit during collection.",
        ),
        Line::raw(""),
        labeled_bullet(
            "Collection:",
            "When an allocation fails, the collector marks everything reachable from the VM's registers, then sweeps the heap and merges neighboring free blocks back into the free lists.",
        ),
        Line::raw(""),
        section("Results"),
        Line::raw(
            "Smallest total VM memory each benchmark needed, without and with the collector:",
        ),
        Line::raw(""),
        Line::raw("  Benchmark        No GC      With GC   Reduction"),
        Line::raw("  queens 15      58.5 MB       8.8 KB     ~6,648x"),
        Line::raw("  maze 20        72.5 MB      60.2 KB     ~1,204x"),
        Line::raw("  bignums 1000   26.7 MB      92.8 KB       ~288x"),
        Line::raw("  pascal 200     14.3 MB     310.0 KB        ~46x"),
        Line::raw(""),
        Line::raw(
            "With the default 1 MB of memory, the VM without a collector could only solve the N-queens problem up to N = 7. With the collector it runs N = 15.",
        ),
    ])
}
