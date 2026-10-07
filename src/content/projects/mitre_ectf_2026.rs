//! Project: MITRE eCTF 2026, a secure hardware security module (HSM).

use crate::content::{bullet, heading, labeled_bullet, link, meta, section};
use ratzilla::ratatui::text::{Line, Span, Text};

/// Each `Line` is a paragraph; long ones wrap to fit the content pane. An
/// empty `Line::raw("")` leaves a blank line.
pub fn text() -> Text<'static> {
    Text::from(vec![
        heading("MITRE eCTF 2026 - Hardware Security Module"),
        meta("Team Lead, Tufts University | Jan. 2026 - Apr. 2026 | 21st of 119 teams"),
        Line::from(vec![
            Span::raw("Repo: "),
            link(
                "github.com/JoeAnselPuplava/2026-ectf-tufts",
                "https://github.com/JoeAnselPuplava/2026-ectf-tufts",
            ),
        ]),
        Line::raw(""),
        Line::raw(
            "MITRE's Embedded Capture the Flag is a two-phase competition. Teams first design and build a secure embedded system, then spend the second phase attacking each other's designs. In 2026 the system was a Hardware Security Module (HSM): a device that stores files and transfers them to other HSMs, with access controlled by a PIN and per-group permissions.",
        ),
        Line::raw(""),
        Line::raw(
            "I led a team of five students. Our firmware is written in C and runs bare-metal on a TI MSPM0L2228 (ARM Cortex-M0+), using wolfSSL for cryptography.",
        ),
        Line::raw(""),
        section("Design"),
        labeled_bullet(
            "Access control:",
            "Permissions are enforced with keys. Each permission group has its own ECC key pairs, and an HSM is only provisioned with the keys for the permissions it has. An HSM without read access to a group never holds the key that decrypts that group's files.",
        ),
        labeled_bullet(
            "File encryption:",
            "Every file is encrypted with AES-CBC under a fresh random key. That key is then wrapped with the group's ECC public key: the HSM generates an ephemeral ECC key, does ECDH with the group key, and hashes the shared secret with SHA-256. Only an HSM holding the group's private key can unwrap it.",
        ),
        labeled_bullet(
            "File transfers:",
            "Receiving a file uses a challenge-response handshake. The sending HSM issues a challenge with the slot, the file's group and a random nonce, authenticated with AES-CMAC. The receiver proves it has receive permission by signing the challenge with the group's private ECC key. The fresh nonce means a recorded handshake can't be replayed.",
        ),
        labeled_bullet(
            "PIN protection:",
            "PINs are stored only as SHA-256 hashes and checked with a constant-time comparison, so neither a flash dump nor timing reveals the PIN. Every wrong attempt triggers a lockout delay that is kept in flash, so power cycling doesn't skip it.",
        ),
        labeled_bullet(
            "Input validation:",
            "UART reads are bounded by buffer size, and slot numbers are checked on every command.",
        ),
        Line::raw(""),
        section("What I built"),
        bullet(
            "The cryptography layer: ECC key wrapping and unwrapping, ECC signing and verification, AES-CMAC signing and verification, PIN hashing with constant-time compare, and secure memory zeroing",
        ),
        bullet(
            "A driver for the board's hardware random number generator, wired in as wolfSSL's entropy source",
        ),
        bullet(
            "Large-file support: flash writes copied the whole file into one stack buffer, so I rewrote them to write in 256-byte chunks",
        ),
        bullet(
            "Performance work to meet the competition's timing limits, including moving wolfSSL to its SP math backend",
        ),
        bullet(
            "Slot checks before every file read and clearing the UART buffer before every packet",
        ),
        bullet(
            "As team lead I split up the work, ran our weekly meetings, and merged and debugged everyone's code into the final design.",
        ),
        Line::raw(""),
        section("Attack phase"),
        bullet(
            "Conducted vulnerability analysis on opposing designs to identify and exploit security flaws",
        ),
        Line::raw(""),
        section("What I would do differently"),
        bullet(
            "To fit the timing limits I turned off wolfSSL's timing-resistant ECC. That made private-key operations faster but reopened the door to timing side-channel attacks. With more time I would have found the speed somewhere else and kept that protection.",
        ),
        bullet(
            "Interrogate requests are encrypted with one AES key shared by every HSM, so compromising any single HSM exposes it. Per-group keys would have been safer.",
        ),
        bullet(
            "Prepare for the attack phase further in advance by having the technology to perform fault injections.",
        ),
        bullet(
            "Do multiple hashes of the pin because if an adversary were to get access of our hash and hash key, they could actually hash every possibile pin to find ours. By doing multiple hashes, the time spent on each pin increases by the ",
        ),
    ])
}
