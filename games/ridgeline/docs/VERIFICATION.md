# Ridgeline verification

Headless and automated native evidence are recorded separately from hands-on
Omarchy/Wayland acceptance, which has **not** been performed.

## Automated evidence (source build, 27 September 2026)

Environment: x86_64 Linux container, Rust 1.98.1, Xvfb with Mesa software GL.

| Check | Result |
| --- | --- |
| `cargo test -p omarchy-ridgeline` (engine, storage, audio and egui interaction tests) | Pass: 28 unit + 2 replay tests |
| `cargo test -p omarchy-ridgeline --no-default-features` | Pass: 18 unit + 2 replay tests |
| `cargo tree -p omarchy-ridgeline --no-default-features` has no eframe/egui/ecolor/Chess | Pass |
| `strategy-evidence`: reference strategy wins all 10 maps × 2 difficulties | Pass; see [BALANCE.md](BALANCE.md) |
| Stored replays re-run at 1×, and at 2× with uneven frame times, give identical battles | Pass |
| Workspace `cargo fmt --check` and `cargo clippy --workspace --all-targets -D warnings` | Pass |
| `scripts/native-ridgeline.py` dark, light, compact and 200% | Pass (all four) |
| `scripts/native-mouse.py`: fifteen mouse launches and returns | Pass |
| `scripts/native-check.py`: one window across fifteen games | Pass locally **with its Stockfish assertion removed**. Stockfish is not installed in this container; CI runs the unmodified script. |
| Suspended-wave save byte-identical after open/close through the real binary | Pass |

The egui tests drive real input events through `App::draw`. They cover:

- mouse build, rejected road placement, right-click cancel, inspect, upgrade and sell;
- start wave, pause, building during a tactical pause, and paused time not advancing;
- Back to Arcade saving the exact battle;
- keyboard 1–4, arrows, Enter, U, F and Space, and Escape cancelling placement before pausing;
- a resting pointer not overriding the keyboard cursor;
- focus loss and the host input gate pausing and saving, reopen restoring the same battle paused, and a render stall pausing;
- confirmation before replacing an unfinished defence;
- a full First Terrace replay through the app, with its medal and unlocks awarded once;
- an unrecorded saved victory awarded exactly once on reopen;
- a rejected save never written until archived;
- every panel rendering at compact size.

The native script drives the real binary under Xvfb through XTest. It checks:

- a fresh campaign opens on the map list;
- keyboard placement at an exact cell and credit deduction;
- Escape cancelling placement;
- Space starting a wave, then Escape pausing and saving;
- Space and U having no effect while paused, and paused ticks not advancing;
- Ctrl+H returning to the shelf in the same window, and Enter reopening;
- a new process restoring the same save unchanged;
- a resumed wave advancing;
- a corrupt/future save left byte-for-byte untouched.

Screenshots are in `screenshots/`. The 200% capture is downscaled by half for
the repository.

## Presentation pass (27 September 2026)

The interface was redesigned: HUD strip, build and inspector cards, wave
preview, campaign map picker with thumbnails and medals, result screen,
modals, and board art. The rules, save format and replays are unchanged.

| Check | Result |
| --- | --- |
| Ridgeline tests after the redesign | Pass: 28 unit + 2 replay tests |
| Workspace strict Clippy and fmt | Pass |
| `native-ridgeline.py` dark, light, compact, 200% | Pass. Dark then passed six further consecutive runs after the hitch fix below. |
| `native-mouse.py`, fifteen games | Pass |

The dark native run failed once during the pass. Cause: the heavier scene
made one software-rendered Xvfb frame exceed 250 ms. The old clock treated
that as a stall and auto-paused, and the test's Escape then resumed the game.
Frame hitches up to one second now run at most a quarter-second of ticks and
drop the rest. Only longer gaps pause. The replay test now includes a 0.6 s
frame and still produces identical battles. Headless measurement: under 1 ms
of CPU per frame for layout and tessellation, about 34,000 vertices for a
busy Last Pass wave.

Note: in this container, eframe's built-in `--screenshot` readback produced
uniform grey images for every game, including existing ones. The captures above
use X11 window grabs, as the native scripts do. CI's `--screenshot` loop is
unaffected.

## Not yet demonstrated

- Hands-on play on Omarchy/Wayland, including feel, pacing, the 10–20 minute
  session target and difficulty ratings from human players.
- Completing a whole map entirely by mouse and entirely by keyboard in the
  native app. Map completion is shown through the production engine and the
  app's command path; native checks cover the individual controls.
- Arch package installation and upgrade. CI adds a suspended-wave upgrade
  comparison; it was not run locally.
- Audio playback on a real desktop. Cue PCM validity and owned-player shutdown
  are unit-tested.
