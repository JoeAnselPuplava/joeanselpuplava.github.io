//! Project: mark-and-sweep garbage collector.

use crate::content::{bullet, heading, labeled_bullet, meta, section};
use ratzilla::ratatui::text::{Line, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Mark & Sweep Garbage Collector"),
        meta("Compilers (CS 107), Tufts University | Apr. 2026"),
        meta("Source available on request (course project)"),
        Line::raw(""),
        Line::raw(
            "A garbage collector written in C for the virtual machine that runs programs from our class's MiniScala compiler. Before this project the VM never freed memory, so larger programs simply ran out. The collector works inside one fixed block of memory handed to the VM at startup and never calls malloc.",
        ),
        Line::raw(""),
        section("Design"),
        labeled_bullet("Block headers:", "Every block starts with a one-word header that packs the block's tag and size together."),
        labeled_bullet("Allocation:", "Free blocks live in 32 segregated free lists: one for each small size and a final list for everything larger. Allocation starts at the list for the requested size and splits a bigger block when it has to."),
        labeled_bullet("Heap bitmap:", "A bitmap with one bit per heap word sits in front of the heap. It does two jobs. It records which words are the start of a real block, so integers aren't mistaken for pointers, and it doubles as the mark bit during collection."),
        labeled_bullet("Collection:", "When an allocation fails, the collector marks everything reachable from the VM's registers, then sweeps the heap and merges neighboring free blocks back into the free lists."),
        Line::raw(""),
        section("Results"),
        Line::raw("Smallest total VM memory each benchmark needed, without and with the collector:"),
        Line::raw(""),
        Line::raw("  Benchmark        No GC      With GC   Reduction"),
        Line::raw("  queens 15      58.5 MB       8.7 KB     ~6,700x"),
        Line::raw("  maze 20       103.0 MB      60.0 KB     ~1,700x"),
        Line::raw("  pascal 200     14.3 MB     311.0 KB        ~46x"),
        Line::raw("  bignums 1000   26.8 MB       1.3 MB        ~21x"),
        Line::raw(""),
        Line::raw("With the default 1 MB of memory, the VM without a collector could only solve the N-queens problem up to N = 7. With the collector it runs N = 15."),
        Line::raw(""),
        section("What I would improve"),
        bullet("The mark phase is recursive, so a very long linked list could overflow the C stack. An explicit work list would remove that limit."),
        bullet("Testing it later under AddressSanitizer exposed a bug. When sweep merges free blocks it only clears the bitmap bit of the first one, so a stale bit can make a later integer look like a pointer. The fix is to clear the bit of every block absorbed in a merge."),
    ])
}
