# FreeSki requirements snapshot

Source: [GitHub issue #14](https://github.com/tcballard/omarchy-retro-arcade/issues/14).
Issue updated: 2026-09-12T17:32:23Z. Snapshot captured: 2026-09-12.

The issue body below is preserved verbatim for local planning. Check the live
issue for later changes before implementation; reconcile intentional scope
changes with PLAN.md and the acceptance tracker. Numerical defaults remain
proposed. Snapshot checkboxes are historical; current progress belongs in
[VERIFICATION.md](VERIFICATION.md).

---

## Game and motivation

**FreeSki**: an original SkiFree-inspired downhill skiing game for Omarchy Arcade. Carve through a snowy hillside, dodge trees and rocks, launch off ramps and try to survive one more run.

Tom requested FreeSki as an Arcade addition. The name and genre are requested; the detailed modes, rules and numerical values below are proposed v1 defaults to refine through playtesting. Use original artwork, courses, sounds and creature design.

## First playable scope

- Single-player, top-down downhill skiing with a scrolling snowfield, steering, braking, jumps and collisions.
- **Free Ski:** an endless seeded slope, distance records and an optional pursuing snow creature.
- **Slalom:** proposed five authored courses with gates, finish lines, time penalties and local medals.
- Trees, rocks, ramps and clear course boundaries; omit moving NPC skiers from v1.
- Keyboard and mouse gameplay, clickable common actions, local records, pause and resumable runs.
- Defer online play, global leaderboard submission, trick-combo systems, equipment upgrades, weather simulation and course editors.

## Movement and simulation

- The skier travels downhill automatically. Steering changes heading through a bounded turn rate; speed depends on heading, slope acceleration and braking.
- Sideways carving slows descent. Braking provides a reliable low-speed control; no automatic uphill travel or instant position changes.
- Keep acceleration, maximum speed, turn rate, friction and braking in a documented tuning table.
- Use a UI-independent fixed-step engine and logical world coordinates. Window resize, zoom and rendering rate cannot alter physics, obstacle positions or difficulty.
- Camera follows downhill with sufficient look-ahead to reveal obstacles at maximum speed; clamp its framing so the HUD cannot hide the immediate route.
- Use swept collision or equivalent continuous checks to prevent high-speed tunnelling through obstacles or gates.
- A collision deducts one of a proposed three crash allowances, briefly tumbles the skier and resets speed. Recover into a validated nearby clear position with a visible proposed 1.5 seconds of collision protection.
- Count one collision event per crash, even when overlapping multiple obstacles. Protection expires by simulation time and cannot be extended by pause.
- After the third crash, finish the run and show the result. Do not silently restart.
- Under a sustained simulation backlog, pause clearly rather than skipping collision time.

## Jumps and obstacles

- Ramps initiate an automatic jump with a deterministic height/time arc. Steering remains possible with reduced authority; there is no independent jump button in v1.
- Low rocks can be cleared when airborne above their explicit collision height. Trees and the pursuing creature remain dangerous; do not make jumping blanket invulnerability.
- Land on clear snow normally. Landing into an obstacle uses the ordinary crash rule, without applying duplicate landing damage.
- Give ramps, rocks and trees distinct silhouettes and readable shadows. Show skier height through a ground shadow rather than moving the collision reference ambiguously.
- Cosmetic ski tracks and snow spray must never conceal hazards. Bound particle/trail lifetime and memory use.

## Free Ski mode

- Generate a bounded buffer of deterministic slope chunks from a seed, discarding distant chunks after they are no longer needed.
- Each chunk must provide a connected skiable route with sufficient width and turning room for the permitted speed. Reject invalid layouts with bounded retries and a known-safe fallback.
- Increase density/difficulty gradually up to a documented cap; never place hazards inside the visible playfield as a surprise.
- Record maximum downhill distance reached, not total movement, so carving sideways cannot farm distance.
- Proposed creature appearance: after 1,000 metres, with a visible/audio warning before entering view. It pursues through the same world with documented speed and turning limits, without teleporting onto the skier.
- Catching the skier ends the run. Tune chase behaviour to allow meaningful evasion; record playtesting rather than assuming fairness.
- A **Creature chase** toggle is available before starting; keep local records separate for chase enabled/disabled and identify the mode in results.
- No remote score service or account is required.

## Slalom mode

- Five original authored courses, unlocked sequentially by completing the preceding course.
- Gates have two visible poles and an ordered crossing segment. Count a gate only when crossing downhill between its poles, in order.
- Missing a gate adds a proposed five-second penalty once when passing its resolution line. Pole collisions use the normal crash rules.
- Timer starts on the first intentional start/steer action after Ready; finish is the first valid downhill finish crossing after all gates have been counted or resolved as missed.
- Rank by elapsed simulation time plus penalties. Show raw time, missed gates, penalties and final time separately.
- Bronze for a valid finish; Silver/Gold use authored thresholds supported by reference runs and human playtests.
- Resolve physical events in time order. A fatal crash or creature catch at/before a finish event cannot also award a finish. Creature chase is disabled for Slalom.
- Restarts create a fresh attempt; unlocks and personal bests survive. Pause, menus and resume countdowns do not advance time.

## Controls and presentation

| Action | Keyboard | Mouse |
| --- | --- | --- |
| Steer | A/D or Left/Right | Pointer position left/right of skier sets desired heading |
| Brake | S / Down / Space | Hold right button, plus visible brake control |
| Start | Enter | Ready button |
| Pause/resume | Escape | Visible pause/overlay action |
| Restart | Focusable action in pause/results | Restart button |
| Return to Arcade | Ctrl+H | Back to Arcade |

- Mouse steering sets a heading target, never teleports the skier. The latest intentional pointer movement or steering key selects the input source; ignore stationary pointer position during keyboard play.
- Pointer outside the playfield stops changing steering target; focus loss pauses. Returning focus requires deliberate resume.
- Menus/overlays consume input. Dismissal must not steer, brake or start a run underneath; clear held input on pause/switching.
- Follow existing Ctrl+, settings and Ctrl+M audio conventions.
- Snow should feel like snow while adapting to Omarchy themes: engraved course marks, angular evergreen trees, faceted rocks, crisp skier geometry and active-theme accents.
- Reuse Arcade's shared cabinet materials around the playfield. Preserve contrast in both light and dark themes; distinguish gates and hazards by shape as well as colour.
- Compact HUD: distance or time, crash allowances, gate/penalty state and chase warning.
- Original sound for carving, ramps, crashes, gates and the creature warning; mute and reduced-effects options. Important events remain visible when muted.

## Arcade integration and saves

- New Rust library under `games/freeski`, hosted by the existing `ArcadeGame` lifecycle in the same native window, desktop entry and Arch package. Append its shelf entry without reordering existing games.
- Follow [AGENTS.md](https://github.com/tcballard/omarchy-retro-arcade/blob/main/AGENTS.md), [CONTRIBUTING.md](https://github.com/tcballard/omarchy-retro-arcade/blob/main/CONTRIBUTING.md) and [DECISIONS.md](https://github.com/tcballard/omarchy-retro-arcade/blob/main/DECISIONS.md).
- Related: #6 for mouse-friendly navigation. No broader desktop-support commitment or separately downloaded game.
- Store an independent versioned FreeSki save using existing XDG private atomic-storage helpers. Preserve every other game's saves, settings and approved artwork.
- Save mode/options, rules/course/generator versions, seed/RNG and chunk state, skier position/velocity/heading, jump/crash/protection timers, creature state, gates/penalties, distance and elapsed ticks.
- Flush on pause, normal close, shelf exit and results; checkpoint periodically. Restore paused, including during a jump or chase.
- Persist personal bests/unlocks/preferences separately from the active run logically; award results once. New Run confirms before replacing unfinished progress.
- Preserve corrupt/future/incompatible saves; explain recovery before explicit archive/reset. Stop simulation and owned audio/work when leaving.

## Implementation and acceptance

- [ ] Implement deterministic steering/braking, continuous collision, jump clearance and single-event crash recovery.
- [ ] Test gate direction/order, missed-gate penalties, finish precedence, timer boundaries and exactly-once records.
- [ ] Validate a documented corpus of slope seeds for connected traversable routes, safe recovery and bounded generation/memory.
- [ ] Test chase warning/spawn, catch detection, chase-off isolation and pause/resume of pursuit.
- [ ] Each Slalom course has repeatable production-engine completion evidence; medal thresholds and chase difficulty receive human playtesting.
- [ ] Complete mouse-only and keyboard-only flows; verify input handover, overlay isolation, focus loss and compact click targets.
- [ ] Save/reopen faithfully restores mid-jump, crash recovery, active chase and Slalom progress without changing results.
- [ ] Identical tick-stamped input behaves equivalently across render rates; resize does not change simulation.
- [ ] Actual app captures cover light/dark themes, compact windows and 200% scale.
- [ ] Workspace fmt, strict Clippy, relevant tests, native switching and package install/upgrade checks pass. Document hands-on Omarchy/Wayland testing separately from headless evidence.
- [ ] Add shelf/help/About, original asset provenance, rules/controls, verification notes and material integration decisions.

## Definition of done

FreeSki opens directly into an understandable skiing flow inside Arcade, supports satisfying endless runs and five complete Slalom courses, works with mouse and keyboard, and safely preserves local progress and suspended play.
