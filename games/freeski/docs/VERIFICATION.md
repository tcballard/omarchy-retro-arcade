# FreeSki acceptance and evidence

The full local game is implemented: practice, endless terrain, optional creature
pursuit, five Slalom courses, medals, sound and resumable runs. Tyler previously
accepted the revised movement and responded positively to endless skiing. Human
ratings of the new pursuit/course difficulty and final desktop acceptance remain
open. The completion pass below records current checks; earlier sections are
historical evidence for their named revisions. Issue #14 remains open.

## Choosing checks

Use these task routes for the inner development loop. Root
[CONTRIBUTING.md](../../../CONTRIBUTING.md) and the
[CI workflow](../../../.github/workflows/arcade.yml) still define combined gates.
Run those applicable gates after integration; do not rerun unrelated suites merely
because another document changed.

| Changed responsibility | Discriminating evidence |
| --- | --- |
| Pure physics/contact | Named engine regression, then FreeSki headless suite |
| Terrain/window bounds | Deterministic overlap/ID checks and seed corpus in `tests/endless.rs` |
| Save/schema/records | Migration, original retention, record separation, invalid versions and exact continuation in `tests/hardening.rs` |
| Input/lifecycle/session integration | Frontend tests, especially multi-tick crash reset and chunk/render equivalence; real native input and reopen |
| Visuals or new controls | Actual dark/light/compact/200% app captures and readable keyboard/mouse flow |
| Package/install boundary | Committed-source package build, staged/native install and exact save preservation on upgrade, reported separately |
| Docs/plans/instructions only | Content and current-vs-proposed review, local links, referenced commands/symbols and `git diff --check`; report runtime tests as not rerun |

Existing commands, run from the repository root with the project's Rust/native
prerequisites available:

```sh
cargo test -p omarchy-freeski --locked --no-default-features
cargo test -p omarchy-freeski --locked --lib
freeski_evidence=$(mktemp -d /tmp/freeski-evidence.XXXXXX)
cargo run -p omarchy-freeski --locked --no-default-features --example practice-evidence -- "$freeski_evidence/fixtures"
cargo run -p omarchy-freeski --locked --no-default-features --example endless-evidence -- "$freeski_evidence/fixtures"
cargo run -p omarchy-freeski --locked --no-default-features --example completion-evidence -- "$freeski_evidence/fixtures"
```

The examples create synthetic saves through ordinary inputs. They are current
commands. The `replay` example consumes an initial Save, absolute consecutive
input tick numbers, a relative tick limit (1–1,000,000), optional per-tick expected
Sim and optional final expected phase/ticks/distance. Files are limited to 1 MiB;
Ready/Paused initial states resume deliberately. It reports the first divergence
or a compact summary to stdout. See examples/replay.rs for the JSON structures.
The native script takes the release binary, output directory and layout variant;
its `FREESKI_FIXTURES` variable points at the generated fixture directory. See CI
for the full invocation and native/Pillow dependencies. On a Wayland desktop,
unset `WAYLAND_DISPLAY` for Xvfb tests so the real desktop is not selected.

Avoid rediscovering these demonstrated environment conditions:

- The local Rust toolchain has been run through `mise exec rust@stable -- ...`.
  Check what exists on the current host; historical `/tmp` dependency paths may
  have expired. User-local Rust does not satisfy makepkg's package-database checks.
- Run native layout variants sequentially or allocate separate virtual displays;
  simultaneous `xvfb-run -a` startup has raced. Avoid concurrent heavy compilation
  during software-renderer timing acceptance. Do not relax the backlog pause to
  conceal an overloaded test environment.
- A real Stockfish engine is needed for required Chess checks; an engine timeout
  is a concrete failing prerequisite, not a FreeSki test exemption.
- All scripted gameplay uses isolated XDG state. Native layout evidence is not a
  human difficulty rating. An installed release may differ from the development
  binary; identify which executable is being tested.

For new evidence record revision/dirty status, command and dependencies, input
identity, outcome and retained artifact location. A local `/tmp` path alone is
transient. Keep small causal regressions in the repository and retain bulky
captures through CI artifacts. Promote the minimal failing example into a test
before treating the lesson as reusable evidence.

## Acceptance tracker

| ID | Acceptance area | Current evidence / status |
| --- | --- | --- |
| A1 | Movement/braking, continuous collision, jumping and single-event recovery | Practice engine tests pass, including high-speed sweeps, overlapping hazards, descent into a rock and safe recovery at every authored hazard |
| A2 | Gates, penalties, finish ordering and exactly-once records | Ordered downhill gates, one-time five-second misses, pole collision, finish precedence and record/unlock tests pass |
| A3 | Connected seeds, recovery and bounded generation/memory | 128 seeds × 5 km production runs, zero reference crashes; four chunks / 72 obstacles and 240-segment trail cap; recovery corridor and overlap tests pass |
| A4 | Creature warning, spawn, catch and isolation | Warning, bounded/blocked spawn, relative catch, physical terrain, protected recovery, chase-off and causal catch-before-crash tests pass |
| A5 | Five complete courses and calibrated medals/chase | All five courses complete with zero reference crashes/misses; thresholds measured. Human medal/chase calibration pending |
| A6 | Mouse/keyboard flows, handover, overlays, focus and compact targets | Frontend and native scenarios pass; rules-2 movement received positive human feedback; endless feel evaluation pending |
| A7 | Save/reopen during jumps, recovery, pursuit and Slalom | Practice and endless jump/recovery continuation and native restoration pass; unsupported saves retained; pursuit warning/active/recovery/caught and Slalom progress/results also restored exactly |
| A8 | Render-rate equivalence and resize independence | Frontend replay at 30/60/120 Hz and alternating window sizes produces identical tick state; backlog pause tested |
| A9 | Actual theme/size/scale captures | Dark, light, compact, 200% X11 and local Wayland captures inspected |
| A10 | Workspace, native switching, package/upgrade and desktop acceptance | Workspace/build/native/staged reinstall checks pass; user-local Arch package build and extracted-package checks pass; system pacman installation and human difficulty acceptance not claimed |
| A11 | Shelf/help/About, provenance, rules, controls and decisions | All modes have shelf/help/About, original geometry/sound provenance, rules and controls |

Current completion evidence is in the final dated section. The preceding endless
implementation was `13fa3bf`, with documentation in `4dc1f8f`; initial practice was
`acf6286`. Earlier results do not imply acceptance of later features.

## 12 September 2026 implementation evidence

Environment: local x86_64 Omarchy 4.0.3; Rust 1.98.1. Native automation uses
Xvfb and Mesa software rendering. Desktop capture uses the actual Wayland backend.
The Rust toolchain was installed user-locally; Xvfb, Python test dependencies and
Stockfish were held in temporary/user-local test locations, with no privileged
system package installation.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --locked --all-targets -- -D warnings` | Passed |
| `REQUIRE_STOCKFISH=1 cargo test --workspace --locked --all-targets` | Passed with a real Stockfish 17.1 executable; 231 tests including 18 FreeSki tests |
| `cargo test -p omarchy-freeski --locked --no-default-features` | Passed: 12 engine/storage tests; no desktop dependencies |
| `scripts/build.sh` | Passed: optimized Arcade and preserved C++ Pinball engine |
| Pinball `theme-palette`, `theme-path`, `authored-upstream-table` | Passed: all three |
| Desktop entry and script syntax | Passed |
| `scripts/native-freeski.py` | Passed normal controls, focus/help isolation, input ownership, pause/reopen, same-window shelf return and invalid-save retention |
| Production `practice-evidence` example + native fixture reopen | Passed mid-jump and crash-recovery restoration; inactive restored runs remain unchanged |
| `scripts/native-check.py` against staged installation | Passed singleton, eleven games in one window, Stockfish response, existing saves and clean shutdown |
| `scripts/install.sh` staging and reinstall | Passed: one desktop entry, all eleven game licences, FreeSki provenance; suspended mid-jump file unchanged across reinstall and reopen |
| Actual local Wayland launch/capture | Passed rendering and clean capture exit; interactive playtesting remains separate |
| `packaging/build-arch.sh` | Blocked at dependency validation: pacman has no installed `rust>=1.98` package. The user-local toolchain builds the app but does not satisfy the package database. No Arch package was built or installed; staged installation is separate evidence |
| Hands-on human playtest | Pending; no human findings have been invented |

The production reference in [engine tests](../tests/engine.rs) follows authored
waypoints with ordinary steering targets. It finishes in **3,397 ticks / 56.62 s**,
uses all three ramps and takes zero crashes. Separate tests compare uninterrupted
and save-restored trajectories for 400 subsequent ticks during flight/recovery.

Frontend checks in [app.rs](../src/app.rs) cover mouse start/pause/settings/restart,
held brake, focus loss, overlay isolation, new-run records, backlog handling and
render/resize equivalence. [Input tests](../src/input.rs) check intentional source
handover and mandatory release of held controls after suspension.

## Actual app captures

- [Dark skiing](captures/dark.png)
- [Light skiing](captures/light.png)
- [Compact skiing](captures/compact.png)
- [200% skiing](captures/200.png)
- [Restored jump](captures/mid-jump.png)
- [Restored crash recovery](captures/recovery.png)
- [Local Wayland ready screen](captures/wayland.png)

These committed captures belong to the initial practice revision, not the later
endless build. All are native application captures, not mockups. X11 layout captures show release
builds at 1280 × 900, 900 × 760 and 1800 × 1520 (200%). The Wayland compositor chose
a tiled 941 × 1030 window. Review checked unclipped controls, hazard silhouettes,
contrast and separation of HUD from the slope. Paused restore captures intentionally
show the resume overlay; the simulation state is verified separately.

## Findings and practical limits

- Concurrent debug software rendering at 200% during compilation triggered the
  intended backlog pause. An isolated release run passed. This is not a low-end
  performance certification; actual playing latency still needs a human check.
- Stockfish 19 timed out in the existing full-suite engine handshake on this host,
  while its isolated test passed. The full suite passed with Stockfish 17.1.
  No Chess timeout or engine behavior was changed for FreeSki.
- The existing all-game native harness assumed default Chess board colours but
  picked up the real user's theme. It now writes the expected palette into its
  temporary XDG state before making pixel assertions. User theme files are untouched.
- Existing Pinball/SDL output included `triangle area overflow` warnings during
  switching; its native checks and C++ tests passed. This is not FreeSki output.
- This milestone has no sound. Endless skiing, chase and Slalom remain later scope.

## Human playtest record

Record tester, revision, input method, theme, size/scale and desktop/backend.
Assess turn commitment, braking, hazard reaction time, jump and crash readability,
difficulty and resume behavior. Record findings and resulting numerical changes
in [TUNING.md](TUNING.md). Until that evidence exists, milestone 1 is implemented
but not fully accepted.

## Rules 2: first human feedback revision (12 September 2026)

Revision: the `fix(freeski): build speed and turn through full sideways headings`
commit following `cef046c` on `feat/freeski`. Tyler's initial keyboard playtest
requested much higher top speed and turning through 90° instead of leaning.
The resulting parameters and remaining human acceptance are in TUNING.md.

- PASS: workspace formatting, strict Clippy, all 234 workspace tests with
  Stockfish 17.1, and release Rust/native build. FreeSki has 21 frontend/engine/
  storage tests; its separate UI-free build passes all 14 engine/storage tests.
- PASS: production reference finishes in 1,714 ticks / 28.57 seconds with three
  jumps and zero crashes. Tests measure gradual speed buildup, true left/right
  traverses, keyboard release retaining heading, and legacy-save migration.
- PASS: native X11 dark, light, compact and 200% runs: visible start, mouse/keyboard
  handover, full 90° turn and retained heading after release/resume, pause/focus/
  help isolation, same-window shelf switching, close/reopen, mid-jump/recovery
  fixtures and corrupt-file retention. Actual captures were inspected at
  `/tmp/freeski-captures/v2-{dark,light,compact,200}/quarter-turn.png`.
- Native harness correction: at the faster speed its old five-second opening
  entered the tree section during the input test, causing a crash/reset in the
  compact run. The handover sequence now begins on open snow after 2.3 seconds;
  the compact rerun passes. No collision behavior was bypassed or relaxed.
- PASS: updated build opened as a native Wayland window. The actual user's save
  migrated exactly (only rules version changed; restored paused), and its
  original was verified in the migration backup. Revised human feel/readability
  acceptance remains pending.
- Packaging files are unchanged. The previous staged install/upgrade checks
  remain baseline evidence; package build remains blocked by the missing system
  `rust>=1.98` prerequisite documented above. Packaging and unchanged Pinball
  C++ tests were not repeated for this tuning-only revision.


## 13 September 2026 endless terrain and hardening

Implementation: `13fa3bf` on `feat/freeski`. Two GPT-5.6 Sol medium workers ran
in parallel for terrain and engine/storage, followed by parent integration,
source review and independent combined checks. Issue #14's updated timestamp still
matches the requirements snapshot. This completes the requested hardening and
endless terrain pass; it does not complete creature, Slalom or full release scope.

| Check | Result |
| --- | --- |
| Workspace fmt and strict Clippy, all targets | Passed |
| Workspace tests, with Stockfish 17.1 required | 252 passed |
| FreeSki desktop-independent suite | 29 passed |
| Production seed corpus | Seeds 0–127 × 5 km from rest, zero crashes, over 1,000 jumps |
| Tick-stamped input continuation | Exact states after save/reopen and crossing chunk boundaries |
| Render schedule and crash reset | 30/60/120 Hz produce identical states across resize, chunk crossings and released-key crash recovery |
| Rust release and native Pinball build | Passed |
| Pinball C++ theme/path/authored-table tests | All three passed |
| Desktop entry validation | Passed |
| Staged eleven-game native switching | Passed, including singleton, Chess engine response and existing save paths |
| FreeSki native dark/light/compact/200% | Passed mode choice, start, steering, overlays/focus, quarter-turn, shelf switching, save/reopen and invalid-file retention |
| Suspended endless fixtures | Native restore beyond 1,600 m, mid-jump and crash recovery; generated through ordinary production inputs |
| Staged reinstall and reopen | Exact endless mid-jump save retained through reinstall and reopening |
| Existing user save | Schema-1 practice run/records/preferences preserved exactly with new schema fields; original backup verified |
| Omarchy desktop | Updated native Wayland FreeSki window opened; endless hands-on acceptance remains pending |
| Arch package | Blocked by missing system Rust prerequisite; user-local mise Rust builds succeed. Staged installation is separate evidence |

Captures inspected under `/tmp/freeski-captures/endless-{dark,light,compact,200}/`:
`free-ready.png`, `free-long-run.png`, `free-mid-jump.png`, `free-recovery.png`,
`free-skiing.png`. Local logs: `/tmp/freeski-endless-workspace.log`,
`/tmp/freeski-endless-headless.log`, `/tmp/freeski-endless-native-all.log`.
The staged upgrade evidence is `/tmp/freeski-endless-upgrade/{before,after}.png`.
These are local acceptance artifacts; CI uploads its corresponding native captures.

Two simultaneously launched `xvfb-run -a` checks initially selected the same virtual
display and failed window isolation. Reruns on separate display numbers passed;
CI already runs variants sequentially. Existing Pinball SDL triangle warnings are
unchanged, with its native/C++ checks passing.

The replay evidence uses explicit tick-stamped production inputs; no player-facing
replay importer/exporter is shipped. Statistical route checks establish feasibility,
not difficulty calibration. Test several minutes of the endless mode before tuning
its density or beginning creature pursuit.


## 13 September 2026 system design pass

Documentation-only revision: introduced SYSTEM.md and NEXT.md, replaced stale
setup work in PLAN.md, routed AGENTS.md by task, and distinguished active rules,
historical tuning and revision-specific evidence. The next planned work is shared
session/reproduction support followed by optional pursuit; no proposed interface
or gameplay behavior was implemented in this pass.

Checked local Markdown links/anchors, referenced current commands and symbols,
source consistency, scope/status language and whitespace. Runtime tests and native
captures were not rerun because executable code and assets did not change; the
252-test runtime result above remains evidence for `13fa3bf`, not a new run.


## 13 September 2026 — whole-game completion

The implementation pass adds Session, optional physical pursuit, five-course
Slalom Cup, original audio, schema 3 migration and production-input replay/fixtures.
Player physics remains rules 2 and terrain generator 1. Exact practice ramp/crash
and endless chunk snapshots were captured independently from `13fa3bf` and are
retained as Session regressions. Chase contact before a predicted crash must not
retain that unrealized crash, recovery position, speed or heading.

Completed checks before the implementation commit:

| Check | Result |
| --- | --- |
| Workspace tests with required Stockfish 17.1 | PASS: 287 tests |
| FreeSki without UI | PASS: 59 tests |
| FreeSki UI/input/audio tests | PASS: 17 tests, including three new complete-mode flows |
| Strict workspace Clippy / formatting | PASS |
| Release Rust/native build | PASS |
| Pinball theme/path/authored table | PASS: all three checks |
| Desktop entry validation | PASS |
| Bounded replay CLI | PASS: 120-tick chase continuation; intentional mismatch reports first divergence at tick 1,660 |
| Production fixture generation | PASS: practice/endless jump/recovery, chase warning/active/recovery/caught, Slalom progress/finish/cup finish |
| Dark/light/compact/200% native controls and exact fixture reopen | PASS |
| Staged eleven-game native switching | PASS: singleton, all games, required Stockfish response, existing saves and clean shutdown |
| Staged reinstall | PASS: pursuit, Slalom progress and cup results retained byte for byte before/after reinstall and reopen |
| Human difficulty and Omarchy playtest | Pending; automated reference routes are not human acceptance |

Pursuit and course reference measurements are in TUNING.md. Native automation
uses isolated XDG state and a disconnected audio server; waveform bounds and
owned process cleanup are tested separately. No actual player save is used as a
fixture. CI generates all twelve fixtures and exercises all four layouts, then
checks an active-pursuit save across package reinstall.

Transient local artifacts: `/tmp/freeski-complete-fixtures`,
`/tmp/freeski-complete-native`, and `/tmp/freeski-complete-*.log`. The tracked tests,
examples and CI commands reproduce the evidence after these files expire.


### Long-escape save regression

Review after `80d8d7f` found that a validation-only 512 m pursuit separation limit
could reject a legitimate escape. Seed 17 with ordinary reference steering
reproduced the old failure at tick 7,687, separation 512.126 m. The bounded union
of two terrain windows already supports arbitrary separation within world bounds;
no extra gap limit belongs in persistence. The fix removes that limit and runs
12,000 production ticks with per-tick validation, then saves and restores exactly.
The same regression was run against the old validator to confirm that it fails.

The full native dark/light/compact/200% runs passed on `80d8d7f`. A virtual-display
startup collision on reusing display 182 failed before launching the app; distinct
displays 183 and 184 resolved it without changing the game or timing thresholds.
Installed switching and three staged upgrade cases also passed. The existing
Pinball SDL triangle warnings remain unchanged, with its native/C++ checks passing.

The standard Arch wrapper reached dependency validation and stopped because
pacman has no `rust>=1.98` package. A user-local build of the same PKGBUILD uses
mise Rust, verified remaining dependencies and checksummed sources; its final
result is recorded below. System packages and pacman installation state are not
changed by these checks.

### Final completion closeout — code `4f61399`

| Check | Result |
| --- | --- |
| FreeSki all targets after long-escape fix | PASS: 75 tests |
| FreeSki without UI | PASS: 60 tests |
| Workspace Clippy / formatting and rebuilt release | PASS |
| Package's required workspace tests | PASS: 288, with the package's built, pinned Stockfish |
| Package's Pinball checks | PASS: all three |
| Native packaged app | PASS: all eleven games, singleton, engine response, original save paths and clean shutdown |
| Packaged FreeSki | PASS: normal controls, mode choices, focus/overlays, exact fixture restore, shelf return, mute and future-file retention |
| Package re-extraction and reopen | PASS: exact active pursuit, Slalom progress and completed-cup saves retained |
| Actual Omarchy/Wayland launch | PASS: updated game opened; every legacy field in the player's schema-2 save remained exact, with an original schema-2 backup |
| Human difficulty / audio judgement | Not claimed; new modes await Tyler's playtest |
| System pacman install/upgrade | Not performed; package extraction/re-extraction is the demonstrated boundary |

The main package was built from committed source `4f61399` using the unmodified
PKGBUILD, with a computed checksum replacing its source placeholder. All sources
passed SHA-256 verification. All runtime/build dependencies except the pacman Rust
entry were verified installed. `mise` supplied Rust 1.98.1; `makepkg --nodeps`
bypassed that package-database prerequisite only. `--noextract` reused prepared
committed source and compiler/engine build caches. No build/check phase was skipped.
No system packages, identities or configuration were changed.

Artifact: `dist/arch/omarchy-retro-arcade-0.1.0-1-x86_64.pkg.tar.zst` (131,552,099
bytes), SHA-256 `48eb67577ad75eecb2b10c73226be30819f458dc97631ee1328272374aef7302`.
The corresponding app snapshot, pinned Stockfish source, NNUE and exact PKGBUILD
are in `dist/arch/sources`. App-source archive SHA-256:
`411c11171261f3e3f524c9389aa65aeb3c6c430d025068de59d8f56b419ca126`.
The package contains one desktop entry and bundled Stockfish with its licence.

Final artifacts include `/tmp/freeski-complete-package-final.log`,
`/tmp/freeski-package-native-all.log`, `/tmp/freeski-package-native-freeski.log`,
`/tmp/freeski-package-upgrade.log`, and the `package` capture directory beneath
`/tmp/freeski-complete-native`. A live close-pursuit capture in its `creature`
directory shows the original actor at its physical position, produced from seed
17 through normal Session inputs. These temporary paths may expire.

The old release executable was replaced by the tested completion build on the
actual Wayland desktop. The running player save was never used to generate test
fixtures or reset to obtain screenshots. Next work is scoped in NEXT.md; issue
#14 has not been closed or externally updated by this pass.

### Menu design pass — 2026-09-13

The menu pass replaces cramped default dialogs with padded panels, grouped
secondary actions, a prominent primary action, readable results and preferences,
and two short help pages. Start/mode/course controls now sit on an opaque themed
surface. This is a FreeSki presentation change; movement rules and save schema
are unchanged.

| Check | Result |
| --- | --- |
| Workspace formatting and strict all-target Clippy | PASS |
| Workspace tests, required Stockfish 17.1 | PASS: 289, serial execution |
| Final FreeSki all-target tests | PASS: 76, including compact help/preferences visibility and run isolation |
| Release Rust/native build | PASS |
| Pinball theme/path/authored table and desktop entry | PASS |
| Native menu captures: dark, light, compact, 200% | PASS: all four; start/modes, pause, restart, settings, help, results, recovery and shelf return |
| Full FreeSki native lifecycle with all twelve fixtures | PASS: dark; normal mouse/keyboard, exact restore, focus, mode/pursuit controls and future-save retention |
| Actual Omarchy/Wayland relaunch | PASS: current player's full parsed save unchanged after normal close/reopen |
| Human visual acceptance | Pending Tyler's review of the reopened game |
| Arch package rebuild/install | Not run for this presentation-only pass; the earlier package artifact predates these menus |

The initial parallel workspace run failed the unrelated Chess engine-timeout
assertion. The complete serial run passed with the required real engine. The
menu screenshot driver's first attempt passed floating-point coordinates to X11;
integer coordinates corrected the driver, and all four final runs passed.

Reproduce menu evidence with `scripts/native-freeski-menus.py BINARY OUTPUT
[dark|light|compact|200]` under Xvfb, with Pillow and `FREESKI_FIXTURES` pointing to
the production example's fixtures. Use distinct X displays for concurrent runs.
The existing full lifecycle driver includes the new toolbar coordinates. Native
checks use disposable XDG directories and a disconnected audio server; these are
separate from human Omarchy acceptance. Final temporary captures are in
`/tmp/freeski-menus/{dark,light,compact,200,lifecycle}`, with logs at
`/tmp/freeski-menu-*.log`.

### Compact Arcade-themed menus — 2026-09-14

Tyler's screenshot identified excessive header height and styling inconsistent
with the collection. The Free Ski header is now about 104 logical px rather than
269 px: mode/actions share a row, stats/pursuit share a row, and hints use one
line. Slalom adds course choices at Ready. Dialogs use the shared popup frame,
brass borders, square buttons and smaller typography/padding. Simulation and
save format are unchanged.

- PASS: workspace fmt, strict all-target Clippy and 289 workspace tests with the
  required Stockfish engine, run serially; 18 FreeSki library tests also passed.
- PASS: rebuilt release app; native menu checks at dark/light/compact/200%, with
  actual screenshots inspected for header, dialogs and results.
- PASS: full dark native FreeSki lifecycle and all twelve production save fixtures,
  updated visible-control coordinates, focus/handover, shelf return and restoration.
- PASS: normal close/reopen of the actual Wayland game preserved the complete
  parsed player save. The updated app was left open for Tyler's review.
- NOT RUN: new Arch package/install or unchanged Pinball C++ checks for this
  presentation-only revision. Earlier package evidence predates these changes.

Two initial concurrent menu runs failed the first mode-selection assertion while
workspace tests were active. The subsequent isolated runs passed; the capture
helper now allows 300 ms for overlay painting before saving an image. No game
input or simulation timing was changed to accommodate the driver. Final captures
are under `/tmp/freeski-compact/{dark,light,compact,200,lifecycle}` and logs use
`/tmp/freeski-compact-*.log`. Human acceptance of this revision remains separate.

### Pursuit obstacle-stall fix — 2026-09-14

A Sol medium agent implemented the bounded escape-heading fix and a regression
from the observed rock contact. Parent review and independent replay confirmed
zero movement in 600 old-policy ticks versus 300.86 m after the fix. Each regression
tick checks valid state, maximum movement and obstacle nonpenetration. The original
player save was read for diagnosis and never replaced with test state.

| Check | Result |
| --- | --- |
| Exact observed rock-contact regression | PASS; old policy stalls, corrected policy escapes |
| FreeSki all targets / headless | PASS: 77 / 61 tests |
| Workspace fmt / strict Clippy / tests | PASS: 290 tests with required Stockfish, serial run |
| Independent 33-seed production Session sweep | PASS: valid saves throughout; longest unprotected stationary spell 140 ticks, no permanent stall in sampled runs |
| Production chase/Slalom fixtures and release build | PASS |
| Native active-pursuit continuation | PASS: actor moved 130.2 m during resumed play; exact state retained through shelf return and normal close/reopen |
| Actual Omarchy/Wayland relaunch | PASS: complete parsed player save unchanged; fixed app left open |
| New package/install / unchanged Pinball C++ checks | Not run; previous package artifact predates the fix |
| Human pursuit-balance acceptance | Pending; the fix does not claim a human fairness rating |

Supplemental local diagnostics are `/tmp/freeski-chase-before.log`,
`/tmp/freeski-chase-after.log`, `/tmp/freeski-chase-corpus.log`, and
`/tmp/freeski-chase-native.log`; captures are under `/tmp/freeski-chase-native`.
Temporary probe sources were removed from the repository; the focused regression
remains in `tests/chase.rs`. TUNING.md records corpus bounds and the observed state.
No player movement constants, causal save fields or menus changed in this pass.

### Original scary yeti redesign — 2026-09-14

Replaced the horned runner with hand-authored layered vector artwork in
`src/yeti.rs`: shaggy white fur, hunched shoulders, long clawed arms, a dark
snarling face, red eyes and fangs. Stride amplitude follows actor speed and
reduced effects; saved ticks keep pause frames still. Escape headings can show
the back of its head. The actor's physical position, chase policy, collision
shape, player movement and save fields are unchanged. Asset provenance is in
`assets/README.md`. This pass was implemented and reviewed without subagents.

- PASS: workspace fmt, strict all-target Clippy and all 290 workspace tests,
  run serially with the required real Stockfish engine.
- PASS: release build and native close-pursuit screenshots in dark/light themes,
  compact size and 200% scale. Inspected the actual running artwork at each size.
  The temporary native helper resumed the existing production seed-17 fixture
  through normal controls in disposable XDG directories.
- PASS: full dark native FreeSki lifecycle, all twelve production fixtures,
  steering handover, pause/focus/help isolation, shelf switching, normal reopen
  and invalid-save retention.
- PASS: normal close/reopen of the actual Wayland game retained the complete
  parsed player save. The updated game was left open.
- NOT RUN: new package/install and unchanged Pinball C++ checks; previous package
  evidence predates this presentation revision.
- PENDING: Tyler's visual acceptance. Automated native captures are separate
  from human Omarchy playtesting.

Temporary evidence is under `/tmp/freeski-yeti-native`, including the four close
views and lifecycle captures. Build, Clippy, test and lifecycle logs use the
`/tmp/freeski-yeti-` prefix. The close-up preview is a magnified native screenshot,
not a separate mockup or replacement game state.

### Cohesive FreeSki artwork pass — 2026-09-14

Tyler approved the yeti and requested its level of craft across the game. Original
layered skier poses, snowy trees, faceted rocks, raised ramps, Slalom cloth flags,
finish markers and the shelf SVG now share that direction. Wind marks, directional
tracks, twelve-point powder and one short landing puff add bounded cosmetic motion.
The skier stays upright during yaw, with distinct brake, flight and tumble poses.
The approved yeti geometry is unchanged; common helpers moved into geometry.rs.

- PASS: workspace fmt, strict all-target Clippy and 290 workspace tests with a
  required real Stockfish engine, run serially. The diagnostic-only addition to
  practice-evidence was subsequently executed and checked with Clippy.
- PASS: release app build; original shelf SVG parsed and inspected in the actual
  Arcade shelf. No new library or external artwork dependency.
- PASS: actual native dark/light/compact/200% captures inspected for skier poses,
  tree/rock detail, jump/shadow separation, landing puff, cloth gates and finish
  flags. Ramp approach was inspected at compact and 200% scale.
- PASS: reduced-effects checkbox selected through native controls at compact and
  200% scale, persisted in the disposable save, then resumed for visual inspection.
- PASS: full dark native lifecycle with twelve production fixtures, input handover,
  focus/pause isolation, shelf switching, reopen and invalid-save retention.
- PASS: actual Wayland game closed normally and reopened with the complete parsed
  player save preserved. Updated game left open for Tyler.
- NOT RUN: new package/install or unchanged Pinball C++ checks. Prior package
  artifacts predate this art pass. Human acceptance of the expanded artwork is
  pending; automated captures do not constitute human playtesting.

`practice-evidence` now writes `ramp-approach.json` through ordinary steering in
addition to mid-jump and recovery. `scripts/native-freeski-art.py` takes the release
binary, output directory and dark/light/compact/200 variant. Set FREESKI_FIXTURES to
a directory populated by the practice, endless and completion evidence examples;
run under Xvfb like the existing native driver. It uses disposable XDG directories,
normal inputs, exact fixture restoration checks and graceful close.

Local captures are `/tmp/freeski-art-native/{dark,light,compact,200,close,lifecycle}`;
logs use `/tmp/freeski-art-`. Initial dark/light captures preceded the extra ramp
fixture and reduced-effects clicks. An initial compact driver run failed because
a ctypes mouse coordinate was a float; integer coordinates fixed the harness,
and the full compact rerun and 200% run passed. No gameplay change was made to
accommodate the driver.

### Human playtest record and difficulty research — 2026-09-14

The [human log](PLAYTESTS.md) records Tyler's Practice and separate Free Ski run.
He reports both control methods, collisions, jumps, third-crash termination,
pause/focus handling, presentation, warning and yeti catching working well.
This updates the prior pending artwork acceptance for exercised play. It does
not establish acceptance at every theme/scale or completion of every mode.

- HUMAN OPEN: speed and difficulty are insufficient; possible yeti sticking is
  unconfirmed. Remaining mode, persistence and release checks are listed in the log.
- RESEARCH: [difficulty audit](DIFFICULTY-AUDIT.md) distinguishes historical
  accounts, current source observations and proposed tuning experiments.
- DOCS ONLY: no runtime behavior changed and no new runtime checks were run.
  Documentation links, content and diff were checked before commit.

### Rules 3 difficulty and fast mode — 2026-09-14

Tyler approved the difficulty audit and F-key fast mode. Four GPT-5.6 Sol medium
workers implemented the independent movement/UI, pursuit, terrain and Slalom
areas; the parent integrated storage, replay, native checks and documentation and
reviewed the combined behavior. Rules/schema/generator versions are now 3/4/2.
Approved artwork and steering authority remain unchanged.

- PASS: workspace formatting, strict all-target Clippy, 308 workspace tests with
  real required Stockfish, and 77 headless FreeSki tests.
- PASS: release Arcade and preserved C++ Pinball builds, three Pinball CTest
  checks, and desktop-entry validation.
- PASS: production Practice/endless/pursuit/Slalom fixture generation. All five
  Slalom courses have clean normal and fast reference runs; targets are in TUNING.md.
- PASS: independent optimized `difficulty-evidence` run, 32 seeds × four input
  strategies × two pace selections, up to 200 seconds each. Normal route: 27 caught,
  five surviving; fast route: 32 surviving without crashes. Both fixed-edge
  strategies crashed out in every seed. Maximum unprotected stationary spell:
  132 ticks. These controllers know the terrain; this is not a human escape rate.
- PASS: original rock-stall and overlapping-tree-cusp regressions, physical edge
  pursuit, detour serialization, and protected-gap/terrain contact ordering.
- PASS: migration retains original bytes, old terrain and previous records;
  resumed legacy finishes cannot earn current-rule medals, and old course unlocks
  survive. Fast mode and causal state continue exactly after restoration.
- PASS: native FreeSki controls/lifecycle in dark, light, compact and 200% variants;
  fast-mode screenshots inspected, compact header remains short. Keyboard repeat,
  paused-key isolation and mouse fast-button activation are covered. Twelve
  suspended fixtures restore exactly, including fast Slalom progress/results.
- PASS: final native chase fixture resumed with F fast mode; both actors advanced,
  then pause and shelf return retained the complete resulting state. The final
  dark rerun also exercised the visible mouse fast button.
- PASS: native one-window switching across eleven games, existing-save paths,
  Solitaire continuation and a real Stockfish reply. This checks shared integration;
  it is not new human acceptance of the other games.
- PASS: staged install layout and executable; a disposable copy of Tyler's actual
  old save migrated through the native staged binary with its run, terrain,
  preferences, records and unlocks retained. Reinstall and reopening retained the
  migrated save byte-for-byte. Actual user data was not reset for these checks.
- PASS: actual Wayland app closed normally and reopened with the updated release.
  Compared all prior run/chase fields, terrain identity, records, preferences and
  unlocks after the explicit schema migration. The updated game was left open.
- NOT RUN: a new Arch package build or system pacman install/upgrade. Staged
  installation is separate evidence from installed-release acceptance.
- HUMAN PENDING: difficulty, speed and Slalom acceptance of this revision, tracked
  in PLAYTESTS.md. Earlier approval applies to the previously exercised build.

The corpus is reproducible with `cargo run -p omarchy-freeski --locked
--no-default-features --release --example difficulty-evidence`. Replay schedules
may include `toggle_fast: true` before an input tick, using the same running-only
action as the UI. All fixture generation uses ordinary production inputs.

Local evidence uses `/tmp/freeski-difficulty-`: workspace/headless/Clippy/build
logs, `final-corpus.json`, `fixtures/`, `native/` and `upgrade/`. Compact display
evidence preceded the final pursuit-only refinement; the other display variants
used the final movement and pursuit code. Intermediate checks exposed obsolete
rules-2 timing and permanent-edge-lane assertions; their replacements check the
new behavior, including identical crash outcomes at different rendering rates.


## Rules 3: follow-up human acceptance (14 September 2026)

Tyler reports improved Free Ski feel, successful Practice completion, good Slalom
play with five-second misses, working Free Ski save/resume, a longer pursuit-off
run and satisfactory settings/display checks. Course-by-course coverage and
individual persistence/display variants were not itemized. See
[PLAYTESTS.md](PLAYTESTS.md) for the report and current acceptance table.

Pursuit acceptance remains open: repeated overshooting/circling and a delayed
catch beside a crashed skier were observed. Fast-mode tuck art and a real Arcade
preview screenshot are also requested. This documentation update implements no
fixes and adds no automated or package-install evidence. Prior pending-human
entries describe their historical verification sessions.


## Follow-up fixes: close pursuit, tuck and native preview (14 September 2026)

Implemented the three fixes requested in the follow-up human playtest. The new
pose and screenshot are presentation changes; interception corrects pursuit within
rules 3 without adding fields or migrating records. Human approval of these fixes
is still pending in PLAYTESTS.md.

Diagnosis started with `cargo test -p omarchy-freeski --locked
--no-default-features --test chase close_pursuer -- --nocapture`. The stationary
production Session reproduction failed on the old code after five seconds of
circling, with no obstacles or protection. Competing explanations were recovery
protection, terrain detours and swept-catch failure; the first two were absent
and existing sweep tests passed. The fixed minimum corner speed/turn radius was
the primary cause. The direct-segment regression additionally prevents needless
nearby detours. TUNING.md records the selected policy and rejected fixed lead.

| Check | Result |
| --- | --- |
| Workspace tests, locked/all targets, real required Stockfish | PASS: 311 tests; engine supplied via OMARCHY_CHESS_ENGINE |
| Headless FreeSki | PASS: 80 tests; final strengthened chase-only run passes all 21 chase tests |
| Workspace fmt and strict Clippy/all targets | PASS, including the new fixture example |
| Release native app | PASS |
| Close approaches | PASS: 60 speed/offset/heading cases; stopped-skier reproduction catches within 2.5 s; no catches during protected recovery; braked skier caught within 2.5 s after expiry |
| Production difficulty corpus | PASS: 256 runs over 32 seeds, four strategies and two paces; all normal reference routes caught, all fast reference routes survive; max unprotected stall 47 ticks |
| Native close-chase captures | PASS: dark, light, compact and 200%; actual actor motion, saved paused/caught states, shelf return and close/reopen |
| Native pose captures | PASS: ordinary, fast tuck, reduced-effects tuck and fast-mode braking; visually inspected at actual resolution and magnified for geometry comparison |
| Native shelf preview | PASS: real gameplay PNG and final crop inspected at 200%; original approved game art retained |
| Native switching | PASS: singleton and one window across all eleven games, real Stockfish response, existing save identities and clean shutdown |
| Existing Pinball checks | PASS: theme-palette, theme-path and authored-upstream-table, using existing unchanged build |
| Desktop entry | PASS: desktop-file-validate |
| Staged install | PASS: scripts/install.sh into disposable /tmp/freeski-close-stage; not a system package installation |
| Actual Omarchy reopen | PASS: closed previous app normally, reopened updated Wayland build and verified exact retained run/chase/records/preferences/unlocks; game left open |
| New Arch package / system install and upgrade | NOT RUN in this fix pass; remains a separate release acceptance check |
| Human acceptance | PENDING focused retest of pursuit, tuck and preview; previous playtest results preserved |

The very-close capture fixture can reach Caught before a wall-clock pause command;
that is a valid terminal capture, not proof of paused continuation. Dark/light
runs also demonstrated an active paused chase. A first 200% run during concurrent
software rendering hit the deliberate backlog pause before actor motion; the
isolated rerun after render settling passed. Early workspace execution used an
unrecognized engine environment name and correctly failed the required Stockfish
test; the complete rerun with OMARCHY_CHESS_ENGINE passed. Native Pinball emitted
its existing SDL triangle warnings while the switching assertions passed.

Reproduce the source frame with `cargo run -p omarchy-freeski --locked
--no-default-features --example preview-evidence -- /tmp/freeski-close-preview.json`.
The example also writes matched `pose-normal.json` and `pose-fast.json` through
ordinary ticks/toggles. Run `scripts/native-freeski-close.py BINARY OUTPUT FIXTURE
[dark|light|compact|200]` under Xvfb for real native capture/lifecycle evidence. It
uses a disposable XDG directory and never edits actor state or the user's save.
The checked-in shelf source frame predates a catch and its provenance is in
assets/README.md. Capturing later frames does not manufacture a screenshot.

Temporary local evidence: `/tmp/freeski-close-{workspace,tests,final-chase,
final-clippy,build,native-all,install}.log`, `/tmp/freeski-close-corpus.json`,
`/tmp/freeski-close-native/`, `/tmp/freeski-close-poses/` and
`/tmp/freeski-close-wayland.log`. These may expire; tests, fixture generator,
capture script and the shelf PNG are retained in the repository.


## PR preparation and final sign-off

Tyler accepted the final game after delivery of 5458784; PLAYTESTS.md records the
sign-off without inventing further runs. Earlier pending-human entries remain
historical. Fetching current main during PR preparation revealed intervening
upstream changes and integration conflicts. The existing 311-test/native/staging
evidence does not claim validation of that future integrated revision. Resolve
those conflicts, rerun combined checks and complete final system-package
installation/upgrade verification before merge/issue closure.

## 2026-09-15: Remove Space braking

At Tyler's request following PR #42 feedback, Space no longer brakes or blocks
input rearming. S/Down, right mouse and the brake button remain available;
standard focused-button activation is unchanged. Updated current help and rules;
the original issue snapshot remains historical.

Passed: both input unit tests, including holding Space across rearming and
pressing/releasing S/Down; workspace formatting; FreeSki all-targets Clippy with
warnings denied; Arcade release build; diff whitespace check. No new native
desktop playtest or packaging run for this input mapping change.


## 2026-09-15: Integrated package and review readiness

Runtime revision `a3d0bd77a4a0dd22a9dbb451e82a3c4c7f87e0bc` integrates main
`5d5c085` with thirteen games. Subsequent changes update documentation and the
native test harness only; the installed player runtime is unchanged.

- Workspace formatting, strict all-target Clippy and 384 workspace tests passed
  with real Stockfish. FreeSki headless tests passed separately.
- All 19 Pinball CTest checks passed. Upstream 2048 tests and 3,524-move parity
  passed. The 256-run FreeSki corpus retained its established result: 32/32
  normal reference routes caught, 32/32 fast routes surviving 200 seconds,
  maximum unprotected stall 47 ticks. These are automated strategies, not human
  escape rates.
- Native X11 thirteen-game switching, actual Stockfish response and save checks
  passed for both the source binary and installed `/usr/bin` app. The first
  local Chess check timed out during concurrent compilation;
  the isolated full retry passed. Mouse checks passed in dark, light, compact
  and 200% layouts; native Tanks and Shatter checks passed.
- FreeSki dark/light passed in CI. Original compact passed locally after CI
  failed its pursuit-checkbox click. Harness commit `373ab6e` clicks the label
  interior and waits up to two seconds for persisted state, retaining the same
  assertion. Updated compact and 200% runs passed with suspended-run fixtures.
  No reproducible gameplay bug was found; current GitHub results remain the
  authority for the final CI run.
- Built the exact Arch source archive for `a3d0bd7`, validated the lean player
  payload (136.61 MiB), and installed it with pacman as a real 0.1.0-1 to
  0.2.0-1 upgrade. All 16 existing save/config files retained their exact bytes.
  Actual Omarchy Wayland reopening of the installed app preserved both an old
  Solitaire session and the real FreeSki save byte-for-byte.
- Local package build used the existing mise/rustup compiler with makepkg's
  dependency check bypassed because pacman's Rust package was absent. Other
  dependencies and source checksums were checked. The initial build was stopped
  during package CTest after Rust tests passed; remaining engine checks passed
  locally, then makepkg repackaged existing outputs. This is not evidence of an
  uninterrupted clean build. GitHub's Arch job supplies that separate boundary.

Source archive SHA-256:
`46f06a0b291f8917ef8e5b46669d6d0e3182e70bd4d7bc3a49e676353d33767a`.
Installed player package SHA-256:
`ca483adaf555212250fc974b0f3a28d1bec81f4990cf53b4d20e5e47238b6c1d`.

Local evidence is under `/tmp/freeski-pr42/evidence/`; package outputs are under
`~/.cache/arcade-pkg-a3d0bd7/`. These are temporary, not repository artifacts.
[PR #42](https://github.com/tcballard/omarchy-retro-arcade/pull/42) links the
current checks. Historical pending entries above are superseded only for the
checks explicitly recorded here. Tyler's final acceptance remains scoped to
PLAYTESTS.md; no additional per-course medals or human pursuit rates are claimed.
