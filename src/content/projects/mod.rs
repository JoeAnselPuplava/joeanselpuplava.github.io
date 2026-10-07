//! The "Projects" directory: one file per project.
//!
//! To add a project: create `<name>.rs` here (copy one of the others), add
//! `mod <name>;` below, and add an `Entry::page` to `ENTRIES`.

use crate::app::Entry;

mod bad_cryptography;
mod cps_ir;
mod mark_sweep_gc;
mod mitre_ectf_2025;
mod mitre_ectf_2026;
mod portfolio_website;
mod rust_rpc;
mod sir_dentist;

/// The projects, in the order they are listed in the menu.
pub const ENTRIES: &[Entry] = &[
    Entry::page("Portfolio Website", portfolio_website::text),
    Entry::page("MITRE eCTF 2026", mitre_ectf_2026::text),
    Entry::page("Rust RPC Parallel Optimizations", rust_rpc::text),
    Entry::page("Mark & Sweep Garbage Collector", mark_sweep_gc::text),
    Entry::page("CPS-IR Optimizations", cps_ir::text),
    Entry::page("Bad Cryptography", bad_cryptography::text),
    Entry::page("MITRE eCTF 2025", mitre_ectf_2025::text),
    Entry::page("Sir Dentist", sir_dentist::text),
];
