# Shatter verification — 13 September 2026

Implementation branch: `feat/shatter`, based on Arcade main `58ab27e`.
This is an implementation proposal for issue #7, not completed Omarchy acceptance.

## Local evidence

- Workspace Rust tests passed: 241 tests, zero failures at the integration check.
  The final Shatter suite additionally covers simultaneous seam contacts and
  expansion expiry. Stockfish was not forced with REQUIRE_STOCKFISH; do not
  interpret the workspace result as installed-engine acceptance.
- Shatter desktop-feature tests: 20 tests covering engine, storage, PCM cleanup
  and egui input handling. Engine-only tests also run without desktop dependencies.
- All 20 authored levels clear with seed 1 through the production `Run::step`
  method, within the replay's 900-second per-level budget. The paddle controller
  follows balls with an ordinary position target and holds launch/fire; it never
  edits balls, bricks, lives, score or power-ups. This proves repeatable clearance,
  not comfortable human difficulty.
- A 2-unit-grid flood-fill checks that steel does not permanently seal targets.
  Collision tests include swept faces/corners, narrow misses, simultaneous seam
  contacts, two balls against armour, moving paddles and expansion expiry.
- Tests cover final-ball life loss, same-tick clear precedence, power refresh/cap,
  laser absorption, deterministic RNG continuation, 30/60/144 fps equivalence,
  the 250 ms catch-up pause and speed-preserving anti-stall redirection.
- Exact saves round-trip multiball, effects and tick state. Private-file mode,
  corruption/future-version retention, unique recovery archives, record
  deduplication and independent practice state are covered.
- egui event tests check fresh-input gating after resume, stationary pointer
  exclusion, latest input source, opposite keys and letterboxing.
- Workspace strict Clippy and formatting passed. Native debug executable builds;
  its `--help` includes `shatter`. Shell syntax and Python harness syntax pass.

## Not verified locally

- X11 screenshots and actual mouse/keyboard operation: Xvfb is absent. System
  dependency installation failed on container setgroups/setuid permissions.
- `scripts/native-shatter.py` is prepared for CI but has not run locally.
  Existing all-game switching and four-theme/scale screenshot harnesses include
  Shatter; CI results must be checked before claiming they passed.
- Complete release/C++ build, Arch package install/upgrade, and required-Stockfish
  gates remain CI checks. Local Rust compilation is not package verification.
- Hands-on Omarchy/Wayland playtesting is outstanding: all 20 levels, paddle feel,
  sound balance, ball visibility, mouse-only and keyboard-only menus, focus loss,
  dark/light palettes, compact window and 200% scale. No human tuning is claimed.

## Implementation choices requiring playtest review

- Both pointer and keyboard paddle movement are capped at 650 units/second to
  prevent a jump when changing input source.
- Anti-stall selects a visible destructible target using swept visibility. If
  no target is directly visible it redirects inward/upward to leave the corridor.
  That fallback should be evaluated against the requested reachable-target rule.
- Multiball clones start at the source's known-safe position with separated
  velocities. Balls do not collide with one another; no clone is offset into a
  wall or brick. Check whether the initial visual separation feels clear enough.

Do not close #7 until native/package results and hands-on acceptance are recorded.
