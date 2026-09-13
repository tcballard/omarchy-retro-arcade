# Omarchy Circuit: illustrated table, upstream physics

Version 0.5.0 implements the approved charcoal, ivory and sage orbital-machine direction.

## Architecture

OmarchyTable builds authored DatFile records and passes them through the normal upstream loader and component constructors. TBall, TTableLayer, TWall, TFlipper/TFlipperEdge, TPlunger, TBumper, TRamp, TTripwire and TDrain handle movement and collision. The physics integrator and collision algorithms remain upstream code.

CircuitView draws the approved background plate with live mechanisms and displays. The image coordinates map to world coordinates as x = (image_x - 540) / 25 and y = (image_y - 500) / 25. Flipper rendering derives its endpoints from the current engine collision edge. Ball rendering uses the engine position, with the ramp collision layer determining foreground order. No image animation drives physics.

The left ramp consists of triangular surface planes, rising from the entrance, with its own collision mask and physical guard rails. Upstream TRamp changes the ball's surface and collision mask at the entrance/exit. A tripwire at the upper turn awards the ramp shot. It is possible to fall back down the entrance when a shot lacks speed.

The plunger's authored pullback/release interval is 100 ms, allowing consistent contact at this table's rest position. This adjusts component configuration, not the upstream collision solver.

## Rules

Three balls, one player. Bumper hits score 100, with 2,500 for twelve hits. Each of eight targets scores 250; clearing both banks awards 5,000. Orbit shots score 1,000, upper-ramp shots 1,500, slingshots 25. Lamps and sidebar counters reflect these events. Repeated sensor contacts are debounced. New game resets all objectives.

## Artwork and themes

The packaged PNG is a cleaned production plate derived from the user-approved generated concept. It contains static rails, plastics, bumper caps and orbital illustration. Moving flippers and balls, circuit and target lamps, plunger indicator and dot-matrix displays are rendered separately. Material highlights and some decorative lamps are baked into the illustration; this is a fixed-camera 2D renderer, not a full 3D scene.

Green glass and accents follow Omarchy while ivory, chrome and amber retain their material colours. The exact official wordmark is composited at the centre after recolouring and remains unchanged. The artwork provenance is in assets/circuit/README.md.

## Persistence and modes

Circuit settings and high scores remain in the existing per-user Circuit directory. There is no mid-game save/resume. Classic mode retains original DAT loading and mission controls. The earlier standalone prototype remains explicit --experimental, with its saved games preserved. No original Windows resources are included.

## Table/artwork alignment

The authored layout in `CircuitGeometry.h` is the single owner of collision,
launcher and bumper measurements on the unchanged plate. Connected two-sided
rails have round joints and end caps; slings and scoring banks have closed backs.
Passive targets score without powered energy. The launcher sits at the visible
coil head and uses a release window that survives the ball's bounce phase.

Ground balls pass beneath the high bridge; raised balls remain above its deck.
The renderer copies alpha-masked plate sprites over ground balls beneath the
bridge and behind bumper caps, preserving the original artwork without triangle
seams. Ball drawing and physics share the same image-plane mapping.

`authored-table-boundaries` retains the earlier boundary regression's intent with
fixtures on this layout: a round rail endpoint, both slingshot backs, a guide and
a scoring module face. `authored-table-depth` compares frames with and without a
ball under/on the high bridge. Standalone decorative post coordinates and the old
launcher hood/deck layout are superseded by the connected route-tested geometry;
they are not retained as a second competing set of colliders.

See [GEOMETRY.md](GEOMETRY.md) for installed-collider audits, route fixtures,
39 adversarial trajectories and normal-input launch checks. Native controls,
resize and nudge lifecycle tests run in CI as well as engine-level checks.

## Verification

The actual executable runs 180 simulated seconds through a complete three-ball game, with bounds and finite-state assertions. Separate physics shots exercise the ramp, target, orbit and drain. Screenshot checks cover standard and compact windows and an alternate palette. CI repeats the collision tests after Arch package installation and checks the earlier prototype's save across upgrade.

Interactive Hyprland acceptance, listening to audio and subjective play tuning still require a real desktop session. The richer artwork does not imply Space Cadet layout or mission fidelity.
