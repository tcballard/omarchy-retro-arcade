# Arcade verification

## FreeSki integration — 15 September 2026

Merged main `5d5c085` while preserving all thirteen games and the lean package.
Combined workspace checks, native switching and the actual system package
upgrade passed. Existing saves survived installation and native Wayland reopen.
[Current FreeSki evidence](../games/freeski/docs/VERIFICATION.md#2026-09-15-integrated-package-and-review-readiness)
separates current checks, CI status and Tyler's recorded human acceptance.
The dated reports below describe their original revisions.

## FreeSki complete local modes — 13 September 2026

FreeSki is the eleventh game in the source build. The completion pass implements
practice, seeded endless terrain, optional creature pursuit, five Slalom courses,
medals, original sound and resumable state. Native dark/light/compact/200% checks,
eleven-game switching and staged pursuit/Slalom upgrade cases passed. The actual
Wayland launch retained the user's existing schema-2 run and its original bytes.
A user-local Arch package build passed its 288 tests and all engine checks; the
extracted package passed native switching, FreeSki flows and save re-extraction.
This is separate from installing it into the system pacman database.

[FreeSki verification](../games/freeski/docs/VERIFICATION.md) records source
revisions, exact checks, package results and remaining human difficulty acceptance.
The older nine-game preview release below is unchanged.

## Nine-game presentation integration — 12 September 2026

The integrated presentation revision passed all 197 workspace tests, strict Clippy, formatting, the release build, three Pinball engine/theme tests and desktop-entry validation. The local staged installation also contains the nine game licenses and cabinet-art provenance.

[Integrated CI run](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34689873161) passed native one-window traversal of all nine games, saves, Stockfish interaction, Stack controls/focus/pause/resume, Snake controls/saves/layouts and Blast solo/local-match lifecycle. Its Arch job built the complete package with bundled Stockfish, installed one desktop entry, exercised the installed games and preserved the Solitaire session through a reinstall.

All 40 actual application captures (collection plus nine games in dark, light, compact and 200% layouts) were visually reviewed. Review caught and corrected unavailable navigation glyphs and the opening Pinball preview crop. See [presentation](PRESENTATION.md) for the screenshots and final correction build.

This session cannot open a local display socket, so its native GUI evidence comes from GitHub's Linux/X11 runners. The historical local evidence below belongs to the earlier consolidation, not this session.

## Original five-game consolidation: local evidence

- All 120 Rust tests pass across the combined workspace, including real Stockfish communication, save compatibility, deterministic rules, card artwork validation and bounded Pinball frame decoding.
- Formatting and Clippy with warnings denied pass for all workspace targets.
- Original Pinball engine tests pass for a full three-ball game, ramp, target, orbit and drain collisions, finite/bounded ball state, and operation without proprietary DAT resources.
- Native X11/XTest checks confirm a single window across five games, singleton locking, Solitaire draw/save/reopen, original save-directory identities, Pinball launch/flipper input, confirmed return to the shelf, and clean shutdown.
- The optimized release was staged with the real installation script. It contains one desktop entry; all five games open from that staged prefix. The bundled Stockfish answered an e2-e4 move entered through the native board.
- Native renders pass at 1120×860, 900×760, 200% X11 scale and a light Omarchy palette. Actual screenshots are under `docs/screenshots`.
- A five-second local software-renderer probe delivered 277 Pinball frames, averaging 56.7 fps while other builds were running. This is a local throughput measurement, not a latency or real-desktop benchmark.

## Original five-game consolidation: GitHub evidence

- Published all five games as ordinary subdirectories with their complete original Git ancestry.
- [Arch package job](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34657618469/job/103453313089): built the package and bundled Stockfish, passed all 120 Rust tests and Pinball engine tests, installed one desktop entry, exercised all five games in one window, and reinstalled without changing the Solitaire save.
- [Native Linux job](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34658145339/job/103454867412): formatting, strict Clippy, all Rust tests, release build, Pinball engine tests, desktop entry validation, native game switching and screenshot capture passed.
- The first Ubuntu GUI run exposed a test-harness race when focusing an unmapped window. The corrected harness waits for X11 viewability and passed the rerun. Application code is identical between the verified Arch package and that rerun.
- The [package](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34657618469/artifacts/10286164667) and [corresponding source plus engine inputs](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34657618469/artifacts/10286583861) are available as build artifacts.

## Remaining desktop acceptance

- Real Omarchy/Wayland desktop acceptance: sound, fractional scaling, low-end CPU/GPU performance, Pinball input latency and game difficulty.

This is a development preview. Headless Linux verification is not a real Omarchy desktop playtest.

## Host architecture refactor — 19 September 2026

Source under test: `7c0474d134d5b7a24e8240b64646ff636a72e447`, based on
`0aa746357192e488d2c2d077f279a347c1fff6e3`. Environment: Linux x86_64,
Rust/Cargo 1.98.1 (rustc 48a229cea). Reference reviewed: OmaCut
`0948c4615d45ac62727b8c69112178e09781b7a4`. See ARCHITECTURE.md.

Reproduced now:

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --locked --all-targets -- -D warnings`: exit 0.
- `cargo test -p omarchy-retro-arcade --locked`: exit 0, nine passed;
  the ignored Pinball fixture was invoked successfully by its parent test.
  Includes both new save/game-drop/lock-release ordering tests.
- `cargo test --workspace --locked --all-targets`: exit 0. Stockfish was not
  installed and REQUIRE_STOCKFISH was not set, so this does not establish
  real-engine Chess acceptance. No native desktop session was used.
- `cargo metadata --locked --offline --no-deps --format-version 1`: exit 0.
- `git diff --check`: exit 0. README/architecture local Markdown links resolve.
- Whitespace/visibility-normalized comparison against the baseline confirms
  catalogue data and all game constructor/lock acquisition logic were preserved.

Environment limitations and checks not run:

- Native window switching, actual Omarchy/Wayland, package build/install and
  Stockfish-required acceptance were not run here. No installed Omarchy revision
  was tested. Existing CI has these broader gates; none is claimed from source review.
- Installing desktop development dependencies with apt failed due to the
  environment's setgroups/seteuid restrictions. Rust host compilation nevertheless
  succeeded with the available libraries. No permissions workaround was used.
- An initial offline Clippy attempt failed because ascii 1.1.0 was not cached;
  the subsequent ordinary workspace Clippy command above passed.
- Publication was blocked by automatic approval review pending explicit permission
  to push the branch to the public repository. Remote CI evidence is unavailable.

## Shared platform extraction — 19 September 2026

Source under test: `537f81c3fb8116be3daedd3d08173a883e1e25c7`, on merged
host refactor `957fc5d4d2f5b24d8139568c5fbf5743deec5fa9`. Linux x86_64,
Rust 1.98.1. Generic helper bodies and palette behaviour were compared with the
baseline; only the palette's Color32 import changed from eframe's re-export to
the same ecolor type. No registry dependency version changed.

Reproduced now:

- `cargo test --workspace --locked --all-targets`: exit 0. Stockfish was not
  required or available locally; real-engine acceptance remains with CI.

- Workspace fmt, diff whitespace and workspace Clippy (all targets, locked): exit 0.
- `cargo check --workspace --offline`: exit 0; updates the lockfile for the local crate.
- `cargo test -p arcade-platform --locked --all-features`: four passed, exit 0.
- `cargo test -p arcade-platform --locked --no-default-features`: three passed, exit 0.
- No-default-feature tests for Stack, Snake, FreeSki, Shatter and Tanks: exit 0.
- Normal dependency trees for those five games and arcade-platform with default
  features disabled contain no eframe, egui, ecolor or omarchy-chess packages.
- No game manifest outside Chess references omarchy-chess. The host retains its
  real Chess game dependency. Existing Chess theme/storage exports still compile.

Historical evidence, not rerun for this extraction:

- PR #46 workflow run 35438448573 passed all three jobs, including Stockfish-required
  workspace tests, native switching/save/input checks and Arch packaging, for
  head f3e2c4b348b3b24889edf8eb0d747bbd057b0b5a. #46 was merged after these checks.

Remaining acceptance:

- New-branch native rendering/switching and Arch build/install remain CI gates;
  the earlier #46 results do not establish this branch's runtime acceptance.
- Actual Omarchy/Wayland playtesting and a local Stockfish-required run were not
  performed here. No installed Omarchy revision was tested.

## Focus and drag lifecycle — 19 September 2026

Source under test: `2cd4447b6d213c3b3aae11aae5cbfebd45443b8f`, based on merged PR #47
(`d0a7839394f0e93aca20ca56f8255966e560dd27`). Linux x86_64, Rust 1.98.1.

- Workspace fmt and diff whitespace checks: passed.
- `cargo clippy --workspace --locked --all-targets -- -D warnings`: passed.
- `cargo test -p omarchy-retro-arcade -p omarchy-chess -p omarchy-solitaire --locked`: passed. Includes four host focus/input regressions and real Chess/Solitaire drag-cancellation regressions. Existing ignored subprocess fixture remains ignored in the ordinary test run.
- `python3 -m py_compile scripts/native-stack.py`: passed (syntax only).
- Native Stack focus/held-key script was extended; native execution, Stockfish-required checks and Arch packaging remain CI gates for this branch.
- No actual Omarchy/Wayland session or installed Omarchy revision was tested locally.
- Detached audio-cue ownership and Pinball teardown scheduling remain follow-up work; see LIFECYCLE.md.

## FreeSki resume race — 19 September 2026

Source: `1c27d769c0e3105ca3e8386c5aa5c2d27f383851`. The first-frame steering regression fails before the fix and passes after it. All 22 FreeSki library tests pass, including held-key suppression across focus loss and overlays. FreeSki all-target Clippy with warnings denied passed; workspace fmt and native script syntax checks passed. Linux headless egui tests, Rust 1.98.1, debug info and incremental builds disabled.

An initial default-profile build exhausted local storage during linking; cleaned Cargo development outputs and reran the library tests successfully. Native X11 execution and full workspace/package checks await this revision's CI. No live Omarchy/Wayland acceptance is claimed. The native assertion now reports variant and saved run state; the previous CI logs alone cannot prove this was the only cause of the intermittent failure.

## FreeSki native quarter-turn timing

Run 35445431862 failed at native-freeski.py's exact quarter-turn assertion; the same head passed PR run 35445434538. This is later than the previously fixed handover race. The failed assertion did not include saved state, so its precise runtime cause is not established.

The native quarter-turn check now begins on fresh snow and waits up to 12 seconds for the public periodic save to show the exact heading, instead of assuming a 1.1-second key hold supplies sufficient simulation time. Unexpected pauses fail immediately; the key is released in finally. Exact heading and released-heading/position assertions remain. Capture happens after pause to avoid screenshot latency affecting the running simulation. Python syntax and diff checks passed locally; native execution remains CI evidence for this revision.

## Owned sound workers — 19 September 2026

Base: `3f3c867ade5bff0aa53719c2085f320d2a082946`. Reproduced on Linux x86_64,
Rust 1.98.1. Tested changed source identity (SHA-256):

```text
6efe1ec4494fa843092d753b0be686a87ed3000c22c39bf4bfcd900cdda7819b  shared/platform/src/audio.rs
f77010595da041178b7275f6e6e41aed0a57786769de46883545daeab0fba4a0  shared/platform/src/lib.rs
9fa55757887d125d0f5e7b01200683b566763b5a0de517f3e3298c6c140474ae  games/chess/src/sound.rs
f1667f9329b4b3e25c68b7e8e23f66ce61e55dd676a7354a6b52cb3afddcff5b  games/scram/src/sound.rs
9fa55757887d125d0f5e7b01200683b566763b5a0de517f3e3298c6c140474ae  games/invaders/src/sound.rs
6e8ad2a88967c1f5e5d2ee0d3de0de4a89bba105cfee4fa24491482667169c0f  games/scram/Cargo.toml
47d5dc4e350c66aefe349ef3c901fd260043fe93b97a48e25f813629043a2088  games/invaders/Cargo.toml
e80dfd5ae298b22fdf8f4ff3b3b984128378401310142e25024bbaebaacc1ee4  Cargo.lock
```

Reproduced now, exit 0:
- `cargo fmt --all --check` and `git diff --check`.
- `cargo test -p arcade-platform -p omarchy-chess -p omarchy-scram -p omarchy-invaders --all-targets --locked`.
- `cargo clippy -p arcade-platform -p omarchy-chess -p omarchy-scram -p omarchy-invaders --all-targets --locked -- -D warnings`.

New Linux subprocess regressions cover active cancellation/reaping/file removal,
close immediately after enqueue, idle shutdown, missing-player recovery, overlap
suppression, two-second timeout and subsequent playback. They use a temporary
player executable with a real stalled process, not an audio server.

Not run here: actual paplay/PulseAudio output, real Stockfish (the optional local
test returns without it), Xvfb/native game switching, Arch packaging and live
Omarchy desktop acceptance. Full workspace/native/package checks remain CI gates.
Shutdown wakes playback waiting immediately and joins the worker, but OS syscall
latency is not hard-bounded. Existing main CI results do not validate this change.

## Pinball shutdown polling — 19 September 2026

Base: `3f3c867ade5bff0aa53719c2085f320d2a082946`. Linux x86_64, Rust 1.98.1.
Tested runtime source identity:

```text
0dd707be759f2683bdab1af024c003e5bcc439244762036b76f2886276e4c561  arcade/src/main.rs
79888c94a987bca1d3917fec2f3e52c5eff98beb6765b51426b9a8c4db068e63  arcade/src/session.rs
48105d7ced75643bffdf935c64b3c9d81c06ecaa5818840f3b3d890d0ea91312  arcade/src/pinball.rs
```

Reproduced now (exit 0):
- `cargo test -p omarchy-retro-arcade --bin omarchy-retro-arcade --locked`: 14 passed, one fixture marked ignored and explicitly executed by its supervising test in five scenarios.
- `cargo clippy -p omarchy-retro-arcade --all-targets --locked -- -D warnings`.
- `cargo fmt --all --check`; `git diff --check`.

The subprocess supervisor enforces an eight-second deadline per scenario. Full
queues, blocked writes and exited children use incremental shutdown polling;
each poll must return in under 500ms, with frames available while pending. A
cooperative child exits successfully in under one second without being killed.
The forced-destructor fallback is exercised separately. Stalled children consume
the existing two-second grace across polls; pipe reader/writer handles must be
joined and the child reaped before completion. These are regression thresholds,
not a hard OS scheduling guarantee or an Omarchy frame-rate measurement.

An egui host test retains the game and lock across pending shutdown, checks a
quit request cannot be downgraded to home, delays the Close command until ready,
and verifies exactly-once save/game-drop/lock-release ordering.

Not run locally: actual C++ engine/native switching and WM-close acceptance, Arch
packaging and live Omarchy desktop testing. Existing CI must cover native Pinball
controls and render/screenshot exit paths. No physics or save-format changes.


### PR #53: FreeSki native help checkpoint synchronization (2026-09-19)

The pull-request workflow run 35458985597 failed in the compact FreeSki native
check at the help/save equality assertion; push run 35458982793 passed. The
script previously captured the save after only the key helper's fixed delay.
It now requires a persisted Running checkpoint before sending F1, then a
persisted Paused checkpoint before taking the help snapshot. Each wait has a
five-second deadline and reports the action, variant and observed run on timeout.
The exact save-equality assertions for steering and dismissing help remain.
No gameplay, save format or release version changes are included in this fix.

Local checks: Python source compilation and git diff whitespace check passed.
Native execution was not run locally because this environment has no Xvfb or
built Arcade binary; the updated push and pull-request workflows must provide
that evidence. This is not additional hands-on Omarchy/Wayland acceptance.
