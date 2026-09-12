# 2048 verification — 12 September 2026

## Passed locally

- All 208 workspace tests passed with `REQUIRE_STOCKFISH=1 cargo test --workspace --locked --all-targets` before adding the five direct egui interaction tests.
- All 16 2048 tests passed after adding those interaction checks. They cover merge-once cases, the upstream adjacent-128 regression, motion metadata, no-op/RNG behaviour, valid spawns, win/continue/undo, game over, bounded history, state validation, private save round-trip and corrupt/future/oversized-file retention.
- The direct egui tests exercise actual pointer and key events through the production UI: direction buttons, board drag, undo after mouse interaction, pause/help/restart isolation, winning/continuing, credits, reduced motion, reopen and unsaved play after a bad save. These do not create a native OS window.
- All 23 unmodified upstream JavaScript verification groups passed.
- All 3,524 moves in `node games/2048/verify-upstream.cjs` matched the pinned upstream engine's boards, scores, moved flags, merge positions and tile motion coordinates.
- Workspace formatting and strict Clippy passed. The release application and unchanged C++ Pinball worker built successfully.
- All three required Pinball CTests passed (`theme-palette`, `theme-path`, `authored-upstream-table`).
- Package staging installed one desktop entry and the 2048 GPL licence and third-party notice, including both author credits and the upstream MIT terms.

- `git bundle verify games/2048/upstream-history.bundle` passed: complete original history at `3f10becf38aa40a21c1555091f677c272830788a`.

## Not established locally

- Native X11 rendering/input and same-window switching: blocked because this session rejects Unix socket creation (`EPERM`). Xvfb therefore cannot start. The real-window `scripts/native-2048.py` check is wired into CI; it has not been reported as passing locally.
- Visual acceptance at dark/light themes, compact window and 200% scaling: the existing CI capture loop now includes 2048. Local automated egui checks establish interactions, not visual acceptance or an actual screenshot.
- `desktop-file-validate`: executable unavailable locally; existing CI installs and runs it. The desktop file itself is unchanged.
- Clean Arch package build, install and upgrade: not run locally. The existing Arch CI job builds the complete package and now switches through all ten games.
- Hands-on Omarchy/Wayland playtesting: outstanding. Check readable tiles, animation feel during rapid moves, mouse dragging, comfortable controls, focus loss/resume, returning to the shelf, high-DPI/light themes and normal close/reopen.

## Reproduce

Use the workspace gates in CONTRIBUTING.md plus:

```sh
node games/2048/upstream/test-game-logic.js
node games/2048/verify-upstream.cjs
xvfb-run -a -s '-screen 0 1440x1100x24' env LIBGL_ALWAYS_SOFTWARE=1 \
  python3 scripts/native-2048.py target/release/omarchy-retro-arcade /tmp/2048-renders
```

Upstream history, source and licence declarations are preserved unchanged.
The saved-state format is new; standalone Tauri/browser saves are not migrated.
