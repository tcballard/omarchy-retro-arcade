# Minesweeper development verification

Date: 2026-09-16. Base: `5d5c085` (main, v0.2.0 README update).
Platform: Ubuntu 24.04.3, Linux x86_64, rustc 1.98.1. This is not an Omarchy desktop session.

Input identity: `verification.sha256` records the new game sources and integration files. Verify from the repository root with `sha256sum -c games/minesweeper/verification.sha256`.

## Reproduced

| Check | Result | Boundary |
| --- | --- | --- |
| `cargo test -p omarchy-minesweeper --locked -j 2` | Exit 0; 12 tests pass | Rules, persistence and egui input; no native display |
| `cargo test --workspace --all-targets --locked -j 2` | Exit 0; 297 tests pass | Existing environment-dependent tests may skip their external prerequisites; Stockfish-required CI is separate |
| `cargo fmt --all --check` | Exit 0 | Workspace formatting |
| `cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings` | Exit 0 | Workspace lint/build checks |
| `cargo build -p omarchy-retro-arcade --locked -j 2` | Exit 0 | Shared native Rust executable; Pinball worker not built |
| `target/debug/omarchy-retro-arcade --help` and `--version` | Exit 0 | Minesweeper registered in CLI |
| Package manifest source check and native harness syntax | Pass | New documentation/licence sources exist; does not prove package install |

The focused suite checks all difficulties and corner/edge/central openings (480 seeded boards), 150 seeded action sequences, chording with correct and incorrect flags, flood reveal, loss/win immutability, negative counters, invalid state rejection, atomic save roundtrips, record accounting and preservation of rejected files. Egui tests exercise mouse reveal/flag at all board sizes in a 900×760 viewport, keyboard play, menu/pause/input blocking, and exclusion of paused/closed time.

## Not run locally

- Xvfb native switching, native Minesweeper harness, screenshots and real focus/backend checks. Display dependencies were unavailable; apt setup could not complete in this environment. CI now includes the harness and Minesweeper in collection switching and render coverage.
- Full Arch package build/install/upgrade/uninstall, including bundled Stockfish and Pinball. Existing package CI covers these; no local package success is claimed.
- Live Omarchy/Hyprland/Wayland play, light/dark theme changes, display scaling and accessibility inspection. Acceptance remains pending.

## Earlier failures resolved

The initial dependency compilation failed while building a zbus archive; a two-job rebuild completed. Compile/lint findings in the new game were corrected before the passing checks above.

This is a development handoff, not a versioned release or desktop-compatibility claim. No game audio or online service is added. The shelf SVG is an original illustration, not a screenshot.
