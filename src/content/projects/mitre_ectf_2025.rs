//! Project: MITRE eCTF 2025, a satellite TV encoder/decoder.

use crate::content::{bullet, heading, labeled_bullet, link, meta, section};
use ratzilla::ratatui::text::{Line, Span, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("MITRE eCTF 2025 - Satellite TV System"),
        meta("Tufts University | Jan. 2025 - Apr. 2025"),
        Line::from(vec![
            Span::raw("Repo: "),
            link(
                "github.com/JoeAnselPuplava/tufts-2025-ectf-insecure-example",
                "https://github.com/JoeAnselPuplava/tufts-2025-ectf-insecure-example/tree/Final-Branch",
            ),
        ]),
        Line::raw(""),
        Line::raw(
            "MITRE's Embedded Capture the Flag is a two-phase competition. Teams first design and build a secure embedded system, then spend the second phase attacking each other's designs. In 2025 the system was a satellite TV broadcast: an encoder protects TV frames before they are sent up to the satellite, and each subscriber's decoder (a MAX78000FTHR board) must only show the channels it has a valid subscription for.",
        ),
        Line::raw(""),
        Line::raw(
            "Our team had 19 students. The encoder is Python using the cryptography library, and the decoder firmware is C using wolfSSL.",
        ),
        Line::raw(""),
        section("Design"),
        labeled_bullet(
            "Encryption:",
            "Two layers of AES. An outer AES-CBC layer hides each frame's channel and timestamp, and an inner AES-GCM layer encrypts the frame itself with a per-channel key derived using HKDF.",
        ),
        Line::raw(""),
        labeled_bullet(
            "Integrity:",
            "An HMAC-SHA256 over the whole packet is checked before any decryption, so tampered frames are rejected early.",
        ),
        Line::raw(""),
        labeled_bullet(
            "Frame checks:",
            "Frame timestamps must strictly increase, which stops replayed frames, and every frame is checked against its channel's subscription window.",
        ),
        Line::raw(""),
        section("My part"),
        bullet(
            "Merged the decoders that different sub-teams had built into one final design, then debugged it until it built and passed testing before the deadline",
        ),
        Line::raw(""),
        bullet(
            "Built the secrets pipeline that turns the generated secrets into a C header during the Docker build",
        ),
        Line::raw(""),
        bullet(
            "Fixed the replay check, which tracked the last timestamp per channel, so it uses a single timestamp across all channels",
        ),
        Line::raw(""),
        bullet(
            "Fixed an out-of-bounds loop in the channel key lookup that was crashing frame decoding",
        ),
        Line::raw(""),
        section("Attack phase"),
        Line::raw(
            "I focused on cryptographic attacks and captured most of our team's flags. Many came from teams misusing AES: hardcoded keys, reused IVs and nonces, and ECB mode. I later turned these attacks into a hands-on lab, Bad Cryptography, which is also in this Projects folder.",
        ),
    ])
}
