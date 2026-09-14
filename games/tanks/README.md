# Tanks — playable preview

Implementation of [issue #8](https://github.com/tcballard/omarchy-retro-arcade/issues/8)
inside the existing native Arcade window. The engine builds without desktop features;
Arcade enables the egui frontend and appends Tanks to the shelf. Solo Easy/Normal AI,
local two-player turns and protected resumable matches are implemented.

See [player controls](HELP.md) and [current verification](VERIFICATION.md). Original
synthesized sounds, launch recoil/flash, weapon-specific impacts, crater opening,
falling tanks, damage labels, aiming arcs, held controls and a first-run chooser
are implemented. Hands-on Omarchy acceptance is still pending.

## Rules implemented

Rules identifier: `tanks-v1-preview`. These are proposed starting values, not
playtested balance. Keep this preview identifier until the complete game is verified.

- Logical arena: 1000 × 600, positive y down. Terrain has 1001 samples with
  linear interpolation; solid ground extends down from the surface. The minimum
  floor is y=580, leaving a supported state even after repeated craters.
- SplitMix64 seeded generation makes piecewise linear hills and flat starting
  pads centred at x=140 and x=860. It validates height bounds, starting support
  and slopes; after eight rejected candidates it uses a known-valid flat field.
  Hills stay between y=280 and y=480, leaving overhead firing space. This does
  not yet establish human terrain fairness or a tested AI firing route.
- First starter is seeded randomly; subsequent round starters alternate,
  including after draws. Each round samples horizontal wind acceleration from
  integer -20..20 units/s² and keeps it fixed throughout that round.
- One `tick()` advances exactly 1/120 second. Semi-implicit Euler first adds
  wind/gravity to velocity and then advances position. Gravity is 100 units/s².
  Angle is 5..175 degrees counterclockwise from screen-right. Power is 1..100;
  launch speed is `40 + 3.6 * power` units/s, shared by all weapons. The muzzle
  is 20 units from the tank centre along its barrel direction.
- Projectile collision uses a point projectile swept continuously along each
  fixed tick's segment against every crossed linear terrain interval and both
  24 × 14 axis-aligned tank bodies. Earliest contact wins; impacts at an exit
  boundary take priority over a miss. There is no direct-hit damage bonus.
  Side/bottom exits miss; travelling above the screen is legal. Flights end
  after 2400 ticks (20 seconds). Traces store muzzle plus at most 2400 samples.
- Blast reference is the body centre, seven units above support. Damage is
  `floor(max_damage * clamp(1 - distance/radius, 0, 1))`. Shell: radius 45,
  damage 50, unlimited; Heavy: radius 65, damage 70, three shots; Digger:
  radius 85, damage 20, two shots per tank per round. Ammo is charged once on
  commit, including misses. Shooter and opponent use the same damage rules.
- At each affected terrain sample, a crater lowers the surface to the circle's
  lower boundary (capped at the floor). No overhangs or caves are represented.
  Craters do not raise terrain. Samples beyond the radius are unchanged.
- Tank support is the highest terrain point under the entire horizontal 24-unit
  track, including interpolated endpoints. The collision body stays axis-aligned;
  any future visual tilt must not change the world-space angle convention.
  Blast damage uses one pre-impact tank snapshot. After terrain removal, both
  tanks settle directly onto their new supports. Fall damage is
  `floor(max(0, new_support_y - old_support_y - 20) / 2)`.
  Health saturates at zero. Settling and damage are a single bounded operation;
  later animation must not reapply them.
- Movement requests travel one horizontal unit, spending the actual Euclidean
  centre displacement from a 60-unit round budget. A step rejects centre-height
  changes over one unit, endpoint track slopes above 12/24, arena edges and
  overlapping/touching tank footprints. Rejections spend no fuel. Movement
  cannot drop across unsupported gaps; terrain-induced falls happen on impact.
- `Ready` gates every turn. Only `Aiming` accepts move/aim/weapon/fire commands;
  `Flying` accepts ticks only. Aim is retained per tank. A player's old trace
  remains until they commit their next shot. At resolution, both alive hands
  over; one alive wins the round; neither alive draws without awarding a win.
  First to two round wins ends the match. `next_round()` is explicit and resets
  health, fuel, ammunition, aim and traces. Ticks outside flight are no-ops.

Determinism means identical tick-stamped inputs and seed on the same build and
floating-point target. Cross-platform bitwise replay is not promised: launch
trigonometry uses the standard library. There is no wall clock, rendering delta,
window size, global randomness or background worker in this engine. `Clone`
retains complete in-memory state, including the projectile and RNG. The versioned persistence layer serializes this state with floating-point roundtrip
support, validates it before use and uses private atomic storage. Reopening pauses;
AI work restarts from its saved seed without changing its eventual choice.

## Verification

```sh
cargo test -p omarchy-tanks --locked
cargo clippy -p omarchy-tanks --all-targets --locked -- -D warnings
cargo fmt --all --check
```

Tests cover 1000 seeds, continuous terrain/tank collisions, blast edges, direct
and self damage, simultaneous elimination, floor support, fall rounding, legal
movement, ammo, command gating, traces, round/match progression, above-screen
flight, expiry and cloned mid-flight state under different tick batch sizes.
The standalone `tanks-engine` CI job runs this crate without native dependencies;
the existing workspace, native switching and Arch package gates remain in place.

## Native interaction direction

Keep the full side-view battlefield readable around a compact control strip.
Use original Omarchy geometry and the shared cabinet palette. The controls should
make the next action obvious without a separate aiming-mode tutorial:

- Mark the active tank with a small labelled chevron; retain player labels and
  distinct track/body details so ownership is readable without relying on colour.
- Group angle, power and an explicit Fire button together. Show numerical values
  beside the angle control and power gauge; allow barrel dragging, step buttons
  and the issue's Up/Down and Left/Right keyboard controls.
- Allow wheel adjustment only while hovering the power control. Consume that
  event so adjusting power cannot also scroll the surrounding interface.
- Keep the cursor visible and local to the window while aiming. Use normal drag
  capture, with no pointer warping or screen-edge wrapping.
- Keep health, wind, fuel and weapon supply visible. Aiming should show barrel
  direction, not an exact predicted ballistic path. Retain the previous-shot
  trace as the player's useful reference for the next adjustment.
- Present a clear launch, flight, impact and settling sequence. Enable the next
  turn only after its presentation completes and the next player accepts Ready.
  The engine already resolves damage once; visual settling must never calculate
  another damage event or allow input against a partly displayed result.
- Preserve the two-tank solo/local scope, existing shortcut table, three weapons
  and best-of-three rules. Additional players and terrain modes remain deferred.

The native preview implements these controls and a saved 108-tick impact sequence.
Implementation and assets remain original; no third-party game source, art or sounds
are bundled. Native X11 captures are checked separately from Omarchy/Wayland acceptance.

## AI and persistence

Easy searches 54 seeded coarse shots. Normal retains that pass, adds 119 finer
shots and 36 repositioned shots, then each difficulty compares available Heavy
and Digger ammunition at its best shell aim. Each call starts at most 32 candidates
and simulates at most 4096 ticks; the frontend requests 1024 ticks per frame.
Scores reward enemy damage and wins, penalise self-damage/death, and charge for
movement and limited ammunition. Dropping the search cancels it. Results cannot
apply to a changed match, and failed movement never partially changes live state.

The 512 KiB bounded tanks.json contains the match, AI seed, mode/difficulty,
preferences and separately counted solo/local records. It checks rules/schema,
terrain dimensions and bounds, support, resources, phase, projectiles and traces.
Corrupt/future files remain in place and disable writes until explicit archival.
Record observation is idempotent. Original bytes are copied into a new private,
synced archive before reset. No other game data is modified.

## Build, run and acceptance

```sh
cargo build -p omarchy-retro-arcade --locked
./target/debug/omarchy-retro-arcade --game tanks
cargo test -p omarchy-tanks --features desktop --locked
```

Target: Omarchy 4 / Hyprland, native Wayland. Current checks run on headless Linux
with Xvfb; no claim of live Omarchy desktop acceptance. See [verification](VERIFICATION.md)
for exact source inputs, commands, renders and remaining gates.

To roll back this development preview, check out the previous revision and rebuild.
Retain `$XDG_STATE_HOME/omarchy-retro-arcade/tanks.json` (normally under
`~/.local/state`); never delete saves during rollback. The added presentation and
sound fields are optional for old preview saves. Other game save paths are unchanged.

Do not close issue #8 on this preview. Human keyboard/mouse match completion,
AI/terrain fairness and Arch install/upgrade acceptance remain pending.
All new engine/frontend code, synthesized sound and shelf geometry are original
GPL-3.0-or-later work.
