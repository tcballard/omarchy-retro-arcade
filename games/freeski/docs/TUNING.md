# FreeSki tuning register

## Current rules 3 — 2026-09-14 difficulty revision

Tyler approved implementing the difficulty audit, including F-key fast mode. The
following values supersede the historical rules-2 values below. Tyler's follow-up
approves general speed/game feel; close-range pursuit required the correction
below. Post-fix human acceptance remains in [PLAYTESTS.md](PLAYTESTS.md).

| Player parameter | Current value |
| --- | --- |
| Normal / fast cap | 60 / 90 m/s (216 / 324 km/h) |
| Downhill acceleration term | 9 m/s², with existing drag and turning losses |
| Straight buildup from rest | Normal: 487 ticks / 8.12 s; fast: 832 ticks / 13.87 s |
| Leaving fast mode | At least 12 m/s² deceleration while above normal cap; stronger braking retained |
| Turning / airborne authority | Unchanged: 1.6 rad/s / 40% |
| Forward view | Unchanged 84.48 m: 1.41 s at normal cap, 0.94 s at fast cap |
| Fast selection | F or the visible button during running play; persisted, no cooldown |

Generator 2 uses a 13 m reserved route, increased lateral variation and at most
22 objects per 128 m chunk. Both edges contain hazards with separated downhill
bands, leaving a validated recovery fallback. The opening retains 92 m of clear
snow. Windows are bounded at 88 obstacles, or 176 for disjoint actor windows.
Generator 1 remains reproducible for existing active mountains; restart uses 2.

| Pursuit parameter | Current value |
| --- | --- |
| Maximum speed / acceleration | 82 m/s / 16 m/s² |
| Desired speed before cornering | Skier speed + 18 m/s, bounded to 48–82 m/s |
| Turn rate / turning drag | 4 rad/s / 7 m/s² at maximum turning effort |
| Braking | 72 m/s² |
| Interception lead | Separation / 82 m/s, capped at 0.5 s; derived from the physical skier segment |
| Corner speed target | Turn rate × target distance / (2 × max(sin(turn error), 0.05)); zero when facing away |
| Anticipatory probe | 0.75 seconds of speed, bounded to 24–56 m |
| Detour lifetime / minimum commitment | At most 120 ticks / 30 ticks; nearby clear interception can release immediately |
| Contact momentum | 65% retained, with an 18 m/s floor; actual movement still stops at contact |
| Warning / recovery | Unchanged 1,000 m / 180 ticks; protected 14 m gap |

Navigation uses bounded full-circle escape selection at contact, including
obstacle cusps and slope edges. Close contact permits outward travel within a
small numeric boundary tolerance, never travel through the obstacle. Protection
and terrain contacts are ordered by their fractions, so later terrain cannot
override an earlier recovery boundary.

Before the close-range correction, the production `difficulty-evidence` corpus
used 32 seeds, up to 12,000 ticks each,
four input strategies and both speed modes. At ordinary pace, 27/32 perfect-route
runs were caught; 5 survived the full 200 seconds. All 32 fast-route runs survived
without crashes. Straight-input runs all ended in a catch or three crashes; both
fixed-edge strategies crashed out in every seed at either pace. The longest
unprotected stationary spell was 132 ticks (2.2 s), with no permanent stall seen
in this sample. This controller knows the generator's route; these are feasibility
and pressure measurements, not human escape rates or universal guarantees.

### Close-range interception correction, 14 September 2026

Tyler observed overshooting, circling and delayed catches beside a crashed skier.
A minimized production Session with no obstacles, no protection and a stationary
braked skier reproduced circling beyond five seconds: yeti offset (8,-8), speed
60 m/s, heading 0. The old corner policy retained at least 42 m/s while turning
at 1.6 rad/s, a minimum turning radius around 26 m. A close target was therefore
unreachable on that circle. Swept catch tests passed; protection and terrain were
absent, excluding them as the cause of this reproduction.

Distance-based corner speed removes the orbit. Stronger braking and turning plus
a bounded lead address moving crossing approaches. A fixed nonzero lead was
rejected because it could settle ahead of a moving skier; the selected lead
shrinks with separation. Near-target navigation checks the actual approach rather
than an obstacle/edge beyond it. Catch radius, speed cap, acceleration, terrain
sweeps, warning and recovery protection are unchanged. No new saved fields or
record migration: this is a correction within rules 3; restored chases retain
state and use the corrected policy.

All 60 clear approach cases (three skier speeds, five offsets, four initial yeti
headings) catch within nine seconds, including initially facing away. The stationary
Session reproduction catches within 2.5 seconds. Protected recovery remains safe
and catches resume within 2.5 seconds when the skier then brakes after expiry.
The seed-17 normal evasive route now buys 36 ticks over straight descent, replacing
the obsolete one-second expectation. The long-gap save regression uses fast mode
to retain its >512 m escape instead of relying on an ordinary-speed missed chase.

The rerun 256-run corpus catches all 32 normal reference routes within 7.15–14.03
seconds after spawn. All 32 fast reference routes survive 200 seconds with no
crashes. Straight strategies all end in a catch or three crashes; fixed edges all
crash out. Maximum unprotected stationary time is 47 ticks (0.78 s). These are
terrain-aware reference inputs, not human difficulty acceptance or universal
navigation guarantees.

| Slalom course | Normal reference | Fast reference | Gold | Silver |
| --- | --- | --- | --- | --- |
| Pinecone Path | 18.13 s | 17.37 s | 19.17 s | 22.00 s |
| Long Turns | 21.72 s | 20.72 s | 23.00 s | 26.00 s |
| Split Pines | 22.90 s | 21.55 s | 24.00 s | 27.00 s |
| Needle Run | 25.27 s | 23.63 s | 26.00 s | 30.00 s |
| Summit Cup | 27.80 s | 25.85 s | 28.50 s | 32.50 s |

These are production-input reference runs with zero crashes/misses. Fast references
use braking before tight turns. They establish feasibility, not human difficulty.
Course geometry and five-second miss penalties remain unchanged. Earlier times
are retained separately, so new medals use only the current rules' records.

## Historical tuning and observations


Historical baseline: rules 2, generator 1. The **2026-09-12 speed and steering**
and **2026-09-13 endless terrain** sections record that revision, superseded by
rules 3 above. Runtime constants live in engine.rs/endless.rs/render.rs. Earlier
pursuit and Slalom measurements below remain evidence for their named revision.
[NEXT.md](NEXT.md) scopes human calibration.

Tyler's later response to the endless pass was positive overall ("this is good").
No duration, seed, input method or specific variety/difficulty findings were supplied;
this does not close those acceptance questions.

Initial implementation values for practice rules/course v1. Retained as the initial baseline; the first human feedback and rules-2 revision
are recorded below.

| Parameter | Initial value | Reason / evidence |
| --- | --- | --- |
| Simulation tick rate | 60 ticks/s | Production engine; 30/60/120 Hz rendering equivalence check |
| World units / footprint | Metres; skier radius 0.65 m | Shared collision and rendering coordinates |
| Slope / view | 80 m wide; view 96 × 72 m | Reserved clear edge corridors; resize-independent visibility |
| Maximum speed | 22 m/s (79.2 km/h) | Fast initial candidate; human reaction-time calibration pending |
| Acceleration | 6.5 × cos(heading) m/s² | Automatic downhill acceleration |
| Drag / carving | 0.10 × speed + 4.5 × abs(sin(heading)) m/s² | Broad turns reduce speed |
| Braking | Additional 18 m/s² deceleration | Reliably reaches rest in engine tests |
| Turn rate / heading limit | 2.4 rad/s; ±1.35 rad | Bounded turns; no uphill direction |
| Mouse dead zone / sensitivity | 1.8% / 32% of field width | Dead zone around skier; normalized heading target |
| Camera | Skier at 24% of view height | 54.72 m / 2.49 s ahead at maximum speed |
| Jump | 1.2 s; 2.5 m apex; 40% steering | Parabolic arc; reference run clears all three ramp/rock pairs |
| Tree / rock / ramp radii | 1.4 / 1.2 / 1.8 m | Plus skier radius for swept contact |
| Tree / rock collision heights | 8 / 0.65 m | Trees dangerous in air; low rocks clearable |
| Crash allowances | 3 | Initial issue proposal retained |
| Tumble / reset speed | 42 ticks / 0 m/s | Stops motion for readable recovery |
| Recovery clearance | Hazard radius + skier radius + 2 m | Every authored hazard checked for clear recovery |
| Protection | 90 ticks after tumble | 1.5 simulation seconds; pause cannot spend/extend running time |
| Checkpoints / backlog | 300 ticks / pause above 250 ms | Five-second simulation checkpoints; no collision time skipped |
| Practice length | 1,200 m | Reference: 3,397 ticks / 56.62 s, three jumps, zero crashes |
| Cosmetic trails | At most 240 segments, one every 3 moving ticks | Bounded memory; reduced-effects toggle removes tracks |
| Endless / creature / Slalom | Not implemented | Tune in milestones 3–4 |

## Initial evidence and observations

The waypoint reference uses only heading targets and ordinary production ticks.
It completes all six practice sections without crashes and uses all three ramps.
The reference is a reachability check, not a claim that the game feels good.

A concurrent debug software-renderer run at 200% scale triggered the documented
backlog pause while other renders and compilation were active. An isolated release
run passed. Keep this behavior visible; do not hide slow frames by skipping ticks.

## Human tuning session format

Record date, commit, parameter old/new values and units, course/seed, input method,
display conditions, observed problem, evidence and next hypothesis.
### 2026-09-12: speed and steering (rules 2, course 1)

Tyler tested the initial native build (implementation `acf6286`, playtest checkout
`cef046c`) and reported that top speed was much too low. He wanted gradual buildup
to a much higher speed, and A/D to turn the skier through 90° rather than make him
lean. Keyboard input is confirmed by the feedback; exact display size/rate and
elapsed play time were not reported.

| Parameter | Old → new | Reason |
| --- | --- | --- |
| Speed cap | 22 → 50 m/s | 2.27× maximum speed; straight buildup about 9.7 s |
| Linear drag | 0.10 → 0.05 /s | Same initial acceleration, sustained buildup |
| Turn rate / limit | 2.4 → 1.6 rad/s; ±1.35 → ±π/2 rad | Deliberate quarter-turn in about 0.98 s |
| Keyboard release | Return downhill → retain heading | Turning sets direction; opposite key turns back |
| View / skier anchor | 96 × 72 → 96 × 96 m; 24% → 12% | 84.48 m ahead, 1.69 s at new maximum speed |
| Skier geometry | Whole-body rotation → ski yaw and upright body | Sideways traverse reads as a turn; goggles show profile |

Acceleration 6.5 m/s², carving friction, brakes, obstacles, jump arc and crash
rules are unchanged. Production reference: 1,714 ticks / 28.57 s, three jumps,
zero crashes. Rules-1 save migration preserves the attempt and records with a
retained original. This revision still needs Tyler's feel/readability acceptance;
1.69 s is measured visibility, not proof of comfortable human reaction time.

### 2026-09-13: endless terrain and persistence hardening

Tyler reported the revised movement was feeling good and authorized hardening
and endless Free Ski. Speed, acceleration, turning, brake force, collision sizes,
jump arc and recovery timing remain the rules-2 values above.

| Parameter | Value | Evidence / purpose |
| --- | --- | --- |
| Chunk length | 128 m | Whole chunks cached; no per-frame random placement |
| Active window | Previous, current and next two chunks | At most four chunks / 72 obstacles; over 256 m ahead at a boundary |
| Opening | First 92 m clear | Acceleration and time to choose direction |
| Reserved route | 16 m wide, centres within ±12 m | Smooth connected bends; optional ramp/rock pairs within route |
| Recovery corridors | x = ±36 m | Hazard clearance checked through 80 chunks for 128 seeds |
| Density | 8 slots + min(chunk / 3, 8), or 4 in sparse chunks | Caps at chunk 24 / 3,072 m; failed placements reduce density |
| Sparse stretches / jumps | Hash-selected, nominal 1/6 and 1/3 chunks | Seeded variation; opening has no ramp |
| Placement retries | At most 12 per slot | Rejected slot is omitted; no unbounded generation loop |
| Optional jump rock | 16 m after ramp | Corpus uses production jumps; ordinary skiing can steer around |
| IDs and RNG | Stable chunk/slot IDs; pure u64 hash streams | Regeneration independent of visit order and save timing |
| Records | Separate practice and Free Ski records | Replacing an unfinished Free Ski run banks its reached distance |
| Safety limits | Finite y ≤ 2 billion m; ticks < one year | Corrupt-state guards, beyond feasible normal play |

Reference corpus: seeds 0–127, each starting from rest and reaching 5 km with
ordinary production steering, zero crashes and over 1,000 jumps combined. This
checks reachability at capped density, not player comfort or long-term variety.
The public reference steering helper is for evidence only; gameplay never calls it.

A frontend crash regression proved released heading must be resolved per physics
tick, because a crash resets heading between ticks in a single rendered frame.
The same post-crash state now results at 30/60/120 Hz. Endless runs across chunk
boundaries also match across these render schedules and alternating window sizes.

Next human checks: terrain variety after several minutes, readable safe lines at
full speed, braking before ramps, and whether edge corridors make runs too easy.
Creature pursuit and Slalom remain separate future tuning sessions.


### 2026-09-13: pursuit, Slalom and original sound

Player movement and generator values remain unchanged. Pursuit is opt-in and
separate from ordinary Free Ski records. These are measured implementation values;
Tyler has not yet rated the new modes' difficulty.

| Pursuit parameter | Value |
| --- | --- |
| Trigger / warning | 1,000 m / 180 ticks (3 s) |
| Spawn candidates | Four gaps 42–60 m behind, five lateral offsets; 20 candidates |
| Failed spawn retry | 60 ticks; counter saturates at 8, retries remain bounded |
| Radius / catch radius | 0.95 m / 1.60 m combined with skier |
| Maximum speed / acceleration | 56 m/s / 10 m/s² |
| Turn rate / turning drag | 1.05 rad/s / 14 m/s² at maximum turning effort |
| Braking / protected gap | 24 m/s² / 14 m |
| Collision | Swept terrain and relative skier contact; no jump exemption |
| Terrain query | Union of skier and creature chunk windows; at most 144 obstacles |

A production Session with seed 17 from rest emits one warning and spawns at tick 1,660. From
that saved pursuit state, straight descent is caught after 912 ticks (15.2 s),
while the reference carving route remains active through 2,400 ticks (40 s) and
increases separation continuously for at least half a second. This demonstrates
control over pursuit, not human fairness or enjoyment. Tests retain the inputs.

| Slalom course | Length / gates | Clean production time | Gold | Silver |
| --- | --- | --- | --- | --- |
| Pinecone Path | 760 m / 11 | 20.80 s | 23.00 s | 26.00 s |
| Long Turns | 900 m / 12 | 25.27 s | 28.00 s | 31.50 s |
| Split Pines | 1,020 m / 15 | 26.75 s | 29.50 s | 33.50 s |
| Needle Run | 1,140 m / 18 | 29.77 s | 33.00 s | 37.50 s |
| Summit Cup | 1,280 m / 22 | 32.98 s | 36.50 s | 41.50 s |

All five clean runs use normal production physics from rest with zero crashes and
zero misses. Gold allows about 10% over reference time and Silver about 25%; Bronze
requires a valid finish. Misses cost 300 ticks (5 s). Human medal calibration remains
open. Pole radius is defined in world.rs and uses the same collision engine.

Sound is original 22,050 Hz mono 16-bit PCM, each cue shorter than 0.7 seconds.
Carving hiss has lower amplitude than event cues and cannot interrupt one already
playing. No playback time influences simulation. Reduced effects removes tracks
and creature stride animation; warnings, geometry and outcomes remain readable.

### 2026-09-14: creature stalled against a rock

Tyler reported that the creature appeared to stop chasing. The observed run used
seed 1789397975898612373 with pursuit enabled. At the saved result, the skier had
reached 4,127.61 m while the active creature remained at 1,477.81 m with speed
0.0417 m/s. The creature was approximately 2 nanometres outside rock 705's combined
collision radius. Replaying 600 production pursuit ticks from that actor state
produced no measurable movement. This confirms a steering deadlock, rather than
chase being disabled or a missing renderer marker. The terminal run itself correctly
stops all simulation; the diagnostic isolates the already-stalled actor state.

The old avoidance policy only tried headings within 1.05 radians of the direction
to the skier, then retried the blocked direct heading. Near contact, every one of
those candidates can point into the same rock. The contact sweep prevented
penetration but the policy never selected an escape direction.

The corrected policy retains the original five probes, followed by ±1.75 and
±2.35 radian escape probes. Selection and work remain bounded; actor turn rate,
speed, acceleration, swept collision and player physics are unchanged. No causal
fields or save migration are added. Existing saved pursuit resumes under the
corrected policy on its next active simulation tick.

The exact observed actor state now travels 300.86 m in 600 ticks, physically
turning around the rock. A supplemental production Session sweep of seeds 0–31
and the observed seed, each for up to 12,000 ticks or a terminal result, kept every
save valid. The longest consecutive stationary active spell outside protection was
140 ticks (2.33 s); six of 33 runs were caught. This bounded sample demonstrates
recovery from the tested stalls, not universal pathfinding or human balance.
The prior dated pursuit measurements remain evidence for their original revision.

### 2026-09-14: first recorded human difficulty feedback

Tyler exercised Practice and a separate Free Ski run. Controls, obstacles,
crashing, jumping, pause/focus behavior, art and sound received positive feedback.
The warning and yeti catch work well, but speed and difficulty feel insufficient.
The previously accepted 50 m/s cap is therefore open for reconsideration;
steering feel remains approved. A possible recurring yeti stall is uncertain.

The [human log](PLAYTESTS.md) records tested and untested areas. The
[difficulty audit](DIFFICULTY-AUDIT.md) compares original SkiFree accounts with
current behavior and proposes coupled speed/pursuit trials. No tuning values or
gameplay changed in this documentation pass, and no new simulations were run.
