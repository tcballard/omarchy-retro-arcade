# Tanks polish verification

Development handoff, 2026-09-14. Target: Omarchy 4 / Hyprland on native Wayland.
Actually tested: Ubuntu 24.04.3 Linux x86_64, Rust 1.98.1, egui/eframe 0.31.1,
Xvfb with software OpenGL and real XTest keyboard input. No live Omarchy version
was exercised. The community badge is not an approval or acceptance claim.

Base: `f2afd3cf014fe2ee17bbef49b62a9925fab18bdd`. The accompanying
[`polish-inputs.sha256`](polish-inputs.sha256) identifies the implementation,
workspace manifests, native scripts and packaging inputs for this pass.

## Reproduced

- `cargo test --workspace --locked`: 280 top-level tests passed, zero failures,
  one ignored; three additional subprocess test executions also passed. This ran
  before the final backward-compatibility test, display fixes and clean rebuild;
  it is supporting evidence rather than an exact-final-source workspace run.
- `cargo test -p omarchy-tanks --features desktop --locked`: 28 passed on final
  source. Includes old-save defaults, first-run mode selection, impact pause and
  exact reopen, reduced-effects outcome equivalence, original PCM bounds and
  process cleanup, mouse firing, command gating and complete AI matches.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all --check`, `git diff --check`: passed.
- `cargo clean -p omarchy-tanks` followed by
  `cargo build -p omarchy-retro-arcade --locked`: passed. Final Tanks tests and
  native captures were rerun after detecting an older incremental render.
- `python3 scripts/native-tanks.py target/debug/omarchy-retro-arcade`, under Xvfb:
  first-run mode selection, keyboard handover/fire/pause, same-window shelf,
  exact saved mid-flight reopen and rejected-save preservation passed.
- `python3 scripts/tanks-renders.py target/debug/omarchy-retro-arcade OUT`, under
  Xvfb: chooser, aiming, Digger impact and post-impact frames captured in dark,
  light, compact and 200% modes. No game test hooks or imported visual assets.
  Read-only save polling locates the real impact for capture. Reviewed examples:
  [dark aiming](../../docs/polish/tanks-aim.png),
  [Digger impact](../../docs/polish/tanks-impact.png),
  [compact](../../docs/polish/tanks-compact.png),
  [light](../../docs/polish/tanks-light.png),
  [200%](../../docs/polish/tanks-200.png).
- Packaging scripts pass `bash -n`; player manifest has unique destinations and
  existing nonempty Tanks help/licence files. No runtime dependencies added.

## Limits and outstanding acceptance

- This is native X11 evidence, not Wayland/Hyprland acceptance. Human complete
  mouse-only and keyboard-only matches, terrain fairness, AI difficulty and
  benchmark feel still need live desktop playtesting.
- Sound synthesis and child cleanup are tested. These headless runs mute sound;
  audible output and device switching require an actual desktop audio session.
- Full twelve-game native switching and the Arch package build/install/upgrade
  gates were not rerun locally in this pass: this environment lacks the built
  Pinball worker and Stockfish. Existing CI gates remain enabled. The Tanks
  native test verifies its own shelf/return lifecycle without replacing workers.
- Initial compilation caught an integer-to-float conversion error, corrected
  before passing checks. Initial Xvfb setup lacked its keyboard compiler and
  local sockets; local runtime setup and a TCP X display enabled the reproduced
  checks. Those setup failures are not desktop acceptance evidence.

This remains a draft preview for issue #8, not a merged or released build.
