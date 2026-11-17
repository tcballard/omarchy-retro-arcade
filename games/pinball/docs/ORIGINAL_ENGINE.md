# Omarchy Circuit: illustrated table, upstream physics

Version 0.5.0 implements the approved charcoal, ivory and sage orbital-machine direction.

## Architecture

OmarchyTable builds authored DatFile records and passes them through the normal upstream loader and component constructors. TBall, TTableLayer, TWall, TFlipper/TFlipperEdge, TPlunger, TBumper, TRamp, TTripwire and TDrain handle movement and collision. The physics integrator and collision algorithms remain upstream code.

CircuitView draws the approved background plate with live mechanisms and displays. The image coordinates map to world coordinates as x = (image_x - 540) / 25 and y = (image_y - 500) / 25. Flipper rendering derives its endpoints from the current engine collision edge. Ball rendering uses the engine position, with the ramp collision layer determining foreground order. No image animation drives physics.

The left ramp consists of triangular surface planes, rising from the entrance, with its own collision mask and physical guard rails. Upstream TRamp changes the ball's surface and collision mask at the entrance/exit. A tripwire at the upper turn awards the ramp shot. It is possible to fall back down the entrance when a shot lacks speed.

The plunger adds 1.5 units every 25 ms (the reference one unit, scaled), reaching full charge in 1.7 seconds, and keeps a separate 0.75-second release window, allowing consistent contact at this table's rest position. A hold of about 0.7 seconds is the least that carries the ball out of the lane. This adjusts component configuration, not the upstream collision solver.

## Motion constants

Gravity, drag and flipper stroke times follow the Space Cadet reference table (`Doc/.dat dump.txt`), rescaled for this table's larger ball: radius 0.45 world units against 0.30. Space Cadet uses gravity 25·sin(0.5) = 11.99 and drag 0.2; the same fall time measured in ball radii needs 17.98 here, authored as 25·sin(0.804). Flipper strokes are the reference 40 ms up and 80 ms down, with the reference flipper surface (smoothness 0.5, elasticity 0.75) and strike multiplier (0.9, scaled). Walls, bumpers, rebounds and targets use the engine's default surface (0.95, 0.6) as the reference table does, and the one-way gate its livelier 0.9/0.8. Large bumpers fire above 2.5 with a boost of 17 and rebounds above 1 with 18, both scaled by 1.5; targets register above 3, scaled. The flipper pivots stay on the illustrated guides and the tips are placed so the clear centre gap is the reference 2.56 ball diameters.

Three things deliberately differ. Targets stay passive: the reference boost of 5 is not applied, keeping the earlier passive-target decision and its test. Tilt keeps Circuit's held-input grace instead of the reference quarter-second. The plinth, the loop floor and the support shed use a livelier surface (elasticity 0.9), because the solver holds a ball to a creep on any slope gentler than atan(1 − elasticity), which is 22° at 0.6. The tube's extra slope keeps its earlier proportion to table gravity. The upstream ramp code registers a ramp's force field from its final plane only; for the authored table the field covers the whole deck, so the tube's slope and drag are uniform along its length. Imported resources keep the original behaviour.

These values match the reference table's proportions; they do not make Circuit a Space Cadet layout, and hands-on feel remains a desktop acceptance item.

## Rules

Three balls, one player. Bumper hits score 100, with 2,500 for twelve hits. Each of eight targets scores 250; clearing both banks awards 5,000. A launch carried through the right orbit scores 1,000 (the orbit lane is the shooter lane, so this is a launch award rather than a flipper shot), upper-ramp shots 1,500, slingshots 25. Lamps and sidebar counters reflect these events. Repeated sensor contacts are debounced. New game resets all objectives.

Each accepted nudge costs 0.3 of the tilt meter and holding adds the rest, so one rested, held nudge still tilts at 1.75 seconds while four rapid taps also tilt; the meter drains in one second per unit. A ball that has rested in open play for 6.5 seconds, away from the plunger and with no flipper raised, receives a small upward kick (a ball search) and the display says so. The upstream Space Cadet rescue stays disabled because it refers to that table's components.

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
