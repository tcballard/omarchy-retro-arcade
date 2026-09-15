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
