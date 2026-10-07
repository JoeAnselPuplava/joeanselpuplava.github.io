//! Project: Bad Cryptography, an educational lab on crypto failures.

use crate::content::{heading, labeled_bullet, link, meta, section};
use ratzilla::ratatui::text::{Line, Span, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("Bad Cryptography"),
        meta("Personal Project | May 2025 - Oct. 2025"),
        Line::from(vec![
            Span::raw("Repo: "),
            link(
                "github.com/JoeAnselPuplava/eCTF-Bad-Cryptography",
                "https://github.com/JoeAnselPuplava/eCTF-Bad-Cryptography",
            ),
        ]),
        Line::raw(""),
        Line::raw(
            "A hands-on lab built from real insecure designs that schools submitted to the 2025 eCTF. Each level gives you a decoder that is only subscribed to channel 1, plus encrypted frames from a channel you aren't subscribed to. To get the flag you have to break the design's cryptography and decrypt that channel.",
        ),
        Line::raw(""),
        section("Levels"),
        labeled_bullet("Level 0:", "a warm-up on checking a flag against its SHA-256 hash"),
        labeled_bullet("Level 1:", "frames \"encrypted\" with a single-byte XOR and a hardcoded key"),
        labeled_bullet("Level 2:", "real AES-CBC, but with the key and IV hardcoded into the secrets generator"),
        labeled_bullet("Level 3:", "AES in ECB mode, where encrypted blocks can be swapped between channels"),
        labeled_bullet("Level 4:", "AES-GCM with a fixed nonce, so every frame reuses the same keystream"),
        Line::raw(""),
        Line::raw(
            "Every level includes the decoder's source code (no security by obscurity) and a set of hints that reveal a little more each time.",
        ),
        Line::raw(""),
        section("Takeaway"),
        Line::raw("Using a strong cipher doesn't make a system secure. These designs all used standard algorithms and still fell apart because of how they were used."),
    ])
}
