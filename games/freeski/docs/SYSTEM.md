# FreeSki system design

The 13 September 2026 completion pass implements the shared simulation direction
from the preceding agent-ergonomics review. Live play, replay and full-run evidence
use the same Session. There is no developer-only physics path.

## Navigation and authority

| Question | Source |
| --- | --- |
| What is playable and what is next? | PLAN.md and NEXT.md |
| What does the game do? | RULES.md, checked against production code |
| Why these numbers; what did a person observe? | TUNING.md |
| What was actually demonstrated? | VERIFICATION.md |
| Why this integration? | Root DECISIONS.md |
| What was originally proposed? | REQUIREMENTS.md, historical issue snapshot |

Code is the executable authority. Resolve discrepancies with the rules explicitly.
Do not infer human acceptance from a reference run or reuse a passing result after
changing its relevant source. Keep historical observations dated.

## Ownership

| Responsibility | Owner and contract |
| --- | --- |
| Keys, mouse ownership, focus and overlays | input.rs and app.rs; emit intent and clear held input at lifecycle boundaries |
| Wall-clock accumulator | app.rs; runs fixed ticks, pauses above 250 ms backlog |
| Complete run transition and bounded terrain cache | session.rs::Session; new(Save), refresh(), step(Input) -> Vec<Event> |
| Movement, flight, swept contact and recovery | engine.rs::Sim; StepOutcome exposes physical TickSegment before recovery relocation |
| Generated terrain | endless.rs; pure seed/chunk geometry with stable IDs |
| Slalom geometry and objective accounting | course.rs; five authored courses, ordered physical crossings and penalties |
| Pursuer policy | chase.rs; deterministic two-phase tick plan, bounded movement, swept relative catch |
| Records and replacement | storage.rs::Save; separate domains, exactly-once results |
| Decode, migration, retained originals and atomic write | storage.rs; bounded validation before reconstructing terrain |
| Drawing and trails | render.rs projects state; artwork.rs/yeti.rs draw sprites with geometry.rs; app.rs owns bounded tracks and one landing puff |
| Sound | audio.rs; original PCM cues, at most one owned playback process, stopped/reaped on pause/mute/exit |
| Full-run evidence | examples and integration tests; ordinary inputs through Session |

The native adapter resolves retained keyboard heading against the current state
for every physics tick. A crash within a multi-tick frame therefore cannot
resurrect an old heading. Session refreshes terrain around movement/recovery;
app and examples do not repeat the cache/step orchestration. Direct Sim calls
remain appropriate for focused physics tests.

## State and event contracts

Run identity comprises mode, chase option, seed/course and applicable schema,
rules/course/generator versions. Save owns this identity and every causal field:
skier position, speed, heading, flight, crash/protection timers, tick count, fast-mode selection,
pursuit phase/position/motion/timers and bounded navigation waypoints and ordered Slalom progress. Session stores
one Save and a derived obstacle cache. Tracks, sound, focus and wall time are not
causal state and do not survive restoration.

Paused and terminal sessions do not advance. Engine physics exposes physical
segments and contact fractions. Session compares pursuit contact with skier
contacts before awarding the result. Recovery relocation cannot pass a gate or
become a catch sweep. A crash stops ordinary movement for the rest of that tick.
A catch is a separate terminal outcome; it does not spend a crash allowance.
Slalom uses the same movement engine with its authored finish distance.

Practice, chase-off distance, chase-on distance and each course's best final time
are independent record domains. Restart/mode selection banks applicable unfinished
distance and resets the attempt while preserving records/preferences. Finishing a
Slalom course unlocks the next one once. UI cannot award progress through drawing.

Schema 4 preserves validated schema-1/2/3 and rules-1/2 saves, retaining original
bytes before migration. Earlier records remain separate and readable, unlocks
survive, and a resumed legacy attempt banks only into retained records. New
attempts start the current record domain. Legacy Free Ski without pursuit remains chase-off. Unsupported/corrupt
files remain intact under the explicit recovery flow. Restore pauses a running
attempt without spending timers or changing its causal state.

## Bounded work and repeatable evidence

Generator 2 reconstructs whole chunks beyond the view, with bounded placement
retries and omission fallback. Generator 1 remains available for restored runs;
the terrain cache key includes generator version. Pursuit queries terrain around both actors so a
lagging creature still collides with its world. The cache has an explicit bound;
it never grows with total distance. The 128-seed reference corpus proves sampled
routes, not every player trajectory or a human difficulty rating.

The replay example consumes a bounded JSON file containing an initial Save,
ordered tick inputs and a tick limit. It uses Session and reports a compact result
or the first invalid tick. Fixture examples create practice, endless, pursuit and
Slalom states using production inputs. Same-build/platform exact continuation is
the promise; cross-platform libm bitwise identity is not claimed.

Retain minimal regressions in the repository: released-heading crash reset,
invalid numeric state, old ramp IDs, chunk boundaries, physical catch ordering,
gate direction/misses and save continuation. CI retains native captures. Temporary
local screenshots are evidence for a dated run and may expire; never check in the
user's state directory or reset their run for a screenshot.

When delegation is authorized, divide ownership at session, actor/objective and
presentation boundaries. Agree interfaces early and review the integrated path.
This document does not authorize delegation or choose its backend/model. Use
small discriminating checks while editing and applicable combined gates after
integration. Keep environment prerequisites separate from game rules and report
headless, native, package and human acceptance distinctly.

## Menu presentation

`menu.rs` owns FreeSki panel sizing, typography, spacing, themed controls and stat
cards. `app.rs` owns actions and transitions. Menus inherit Arcade's shared popup
frame and control geometry, with 16 px padding, 32 px action targets, a bounded
440 px content width and viewport-aware vertical overflow. The help guide
separates controls from run types; both pages keep their close action visible at
the supported compact size.

The compact header combines mode/actions and stats/pursuit into two rows, with
a single hint line. Slalom adds course choices only before a run. Shared brass,
ink and ivory materials keep the controls consistent with the cabinet.
