# Circuit collision geometry

`SpaceCadetPinball/CircuitGeometry.h` owns named paths measured on the unchanged 1024 by 1024 playfield. The mapping is `world_x = (pixel_x - 540) / 25`, `world_y = (pixel_y - 500) / 25`; the upstream ball radius is 11.25 artwork pixels. Ball drawing uses this same mapping, without an extra screen-space height displacement.

Physical rails are connected capsule chains: two opposing upstream line faces with radius-offset circular endpoints and joints. This applies to lane dividers, returns, cabinet walls and both tube rails. Obstacles have closed bodies behind their scoring faces. Ground bodies use mask 1; the tube deck uses mask 2. Cabinet and lane rails name both layers explicitly. The target plinth has a bounded footprint and rear bevel; it does not extend to the cabinet roof. Its four upright gaps are narrower than the ball diameter. The rear orbit remains open behind it, and the high tube arch has a ground underpass. Lower tube supports still block ground balls; the entire tube rail remains closed for raised balls. Only numbered target/module faces and the two sling fronts award those scores.

The top-left tube uses sampled smooth cross-sections, with an explicit elevated chute across the pictured landing platform. Its exit returns to ground beyond the platform's front edge. The deck height is `(405 - image_y) * .004` world units, leaving clearance above a ground ball in the high arch. Ground supports resume at artwork y=120. The renderer composites that high deck above ground balls and below raised balls, so traversing the underpass does not look like crossing over the tube. Both its entrance and exit use upstream TRamp portals. Radius-offset rails must leave the centre route traversable at every segment. A short shot may roll back through the entrance; a strong shot must reach the upper exit. Decorative artwork alone is not a collision specification, so these projected height and route choices are authored explicitly.

The launch floor is at the visible coil head (artwork y=873), with the resting ball centred at x=939, y=861.75. The beige arrow strip is traversable launch-lane artwork, not a raised contact plate. The feed starts 32 pixels above the floor. The floor endpoints meet the existing sloped lane walls; the divider continues to the apron. The two floor corners are capped. Its intentional one-way gate sits at the right orbit's exit into the field, not horizontally across the launch lane; an unsuccessful launch may return to the plunger. Return lanes keep enough width for the ball all the way to their drains. The bottom and outlane drains remain intentional loss routes; the launch floor is not a drain.

The integration retains powered bumpers and slings, while stand-up targets and modules score with zero powered boost. The launcher animates real charge and retains a 0.75-second release window, including rapid re-press cancellation.

## Verification

The authored-table runner executes the original engine with public authored data and isolated user settings. It audits installed line/circle colliders and their spatial-grid registration from both sides and each declared layer. Independent landmark rays check scoring faces, obstacle backs, launcher corners, gate direction, tube entrance and the drain. A route sweep rejects a tube centreline blocked by its own radius-offset rails. Independent fixtures in `tests/CircuitRouteFixtures.h` also require 18 ground routes to remain clear: shooter, rear orbit, bridge underpass, upper field, right target return, bumper approaches, left loop, both inlanes/outlanes, both sides of the module bank and slings, main field and drain. Each route is checked against installed upstream colliders and sampled for solid-body/ball-radius overlap. Routes stay within their intended local corridor rather than relying on a detour elsewhere in the table. All four circular bumpers receive 64 circumference probes each. Bumper coordinates live in `CircuitGeometry.h`; the upper-right skirt is centred at (617,210), clearing the target return without reducing its radius.

During simulated play, checks reject nonfinite coordinates, escape from the actual cabinet or tube footprint, entry into solid bodies, wall/bumper penetration beyond a small numerical tolerance, and five seconds stuck away from the plunger. These checks only report test failures; they do not clamp, teleport or rescue production balls. A strong tube shot must actually transition back to ground at the upper exit. The rear-route regression additionally requires six rear-area shots and three real feed/full-charge launches at different contact phases to actually reach the open field. A ball bouncing indefinitely in a small pocket is a failure even if its speed never falls below the old stuck threshold. The geometry sweep covers 39 trajectories across speeds, directions, lanes and raised sections. It runs one process at a time by default; `OMARCHY_GEOMETRY_WORKERS` is an explicit local opt-in to concurrency.

From the repository root, after building:

```sh
OMARCHY_PHYSICS_TEST_TIMEOUT=300 ctest --test-dir build/pinball \
  --output-on-failure -R 'authored-upstream-table|authored-geometry-sweep|authored-rear-routes'
```

For an inspection diagram, run the helper with `OMARCHY_TEST_TICKS=2` and `OMARCHY_TEST_MAP=/path/to/map.json` in an isolated XDG configuration/data directory, with `SDL_VIDEODRIVER=dummy`, `SDL_AUDIODRIVER=dummy`, `--omarchy-table -sw`. Then use:

```sh
python3 games/pinball/tests/render_geometry_map.py map.json \
  games/pinball/assets/circuit/table.png collision-map \
  games/pinball/tests/CircuitRouteFixtures.h
```

The optional map tool needs Pillow and produces ground/raised PNG diagrams from the exported data and an optional numbered route map from the independent fixtures; it never modifies the approved artwork. The native pinball test also enables `OMARCHY_CHECK_GEOMETRY=1` in the real worker and plays a variable-time input sequence. A reported violation fails the test; it never repairs the trajectory. The bridge-layer screenshot test independently verifies that a ground ball is occluded while a raised ball remains visible. These checks complement native gameplay, resizing and user playtesting; they do not establish that every possible trajectory or artwork contour is perfect.

## Tube entrance and low supports

Only the lower mouth promotes a ground ball onto the raised tube. The upper
portal remains an outgoing raised-to-ground transition; a ground-only one-way
rail rejects reverse entry. The low tube side rails are joined across each leg
at the y=120 bridge cut. These two separate caps close the tube bodies without
blocking the ground underpass between them. Raised balls retain the continuous
full-length rails and can traverse both caps on their own collision layer.

Ground-region diagnostics check independently clipped tube segments, including
their interiors, rather than only distance from side rails. The outgoing portal
uses the existing 0.3 artwork-pixel contact tolerance for its layer transition.
`authored-ramp-entry` exercises both support approaches, the reverse upper exit,
the legitimate lower mouth, and three real launch phases. Every recorded
promotion must lie at the lower mouth; launch shots must reach open play.
`OMARCHY_TRACE_RAMP=1` prints entry coordinates only for the authored table.

Standalone route tests use normal fixed-time input release at three charge-start phases. There is no contact-triggered release shortcut in the engine or test environment. Launcher contact and rapid re-press regressions run alongside the geometry checks.
