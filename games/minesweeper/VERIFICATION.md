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

## Visual and desktop-theme pass — 2026-09-16

The refreshed `verification.sha256` identifies this pass. Earlier checks above are historical at b3ef347; they are not reruns against this revision. Reproduced on Ubuntu 24.04.3 / Rust 1.98.1:

- `cargo test -p omarchy-minesweeper --locked -j 2`: exit 0, 14 tests. Added palette precedence, retention and text-contrast checks.
- `cargo fmt --all --check`, workspace Clippy with `-D warnings`, and shared Arcade debug build: exit 0.
- `scripts/native-minesweeper.py target/debug/omarchy-retro-arcade`: exit 0 under Xvfb. Keyboard, pause, same-window shelf, exact resume and rejected-save preservation.
- `scripts/native-minesweeper-theme.py target/debug/omarchy-retro-arcade`: exit 0 under Xvfb. Live dark-to-light switch, invalid-file retention, live fontconfig font switch, unchanged board.
- Inspected real application captures: Intermediate at 1280×900 in dark and light fixture palettes, Expert at 900×760. Temporary XDG profiles; no runtime gameplay/rendering hooks. Two captures reuse the same board. Captures are in `docs/dark.png`, `docs/light.png` and `docs/expert.png` for source review; they are not added to the player package.

Display dependencies were extracted locally for this pass; Xvfb and the app ran in the same process environment over loopback. This resolves the earlier local Xvfb limitation, not the outstanding Arch package or real Omarchy/Wayland acceptance. No whole-workspace test rerun is claimed for this appearance-only pass.

Upstream font contract inspected: omacom/omarchy quattro `bin/omarchy-font-current` blob 839bd4db7b629db34c30988992fe0da0b12925a7 and `bin/omarchy-font-set` blob 080f055b7121d6f66631639318541dbd0d7df33e; fontconfig is canonical.

## Merge preflight — 2026-09-16

CI run [35138151111](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/35138151111) at `7a0caef` passed the Arch package build, installation, package-content verification, bundled Stockfish protocol check, thirteen-game switching, and reinstall/save-preservation checks. Native workspace tests with required Stockfish, Clippy, build, renders and Minesweeper lifecycle/palette/font checks also passed. The native job then failed because the mouse harness still divided the shelf into twelve entries.

The harness now includes Minesweeper and uses thirteen rows; launch/title assertions remain intact. No application runtime code changed. Recaptured and decoded all three review PNGs after finding the committed light image was empty; the dark/light pair shares a board. Repeated the native Minesweeper lifecycle check successfully under local Xvfb. Final CI status is recorded in the PR description. Live Omarchy/Hyprland acceptance remains a release follow-up.
