# SkiFree difficulty audit — 2026-09-14

Implementation status: Tyler approved the first four recommendations on September
14, including F-key fast mode. The subsequent rules-3 revision implements those
changes; see [current tuning](TUNING.md) and [verification](VERIFICATION.md).
The comparisons below describe the pre-change fc9a45a baseline. A second pursuer
and additional game modes remain deferred.

## Finding and evidence limits

Tyler's [human report](PLAYTESTS.md) approves movement controls, collisions,
jumping, presentation and the functioning chase/catch, but finds difficulty and
speed insufficient. Preserve those successes while reopening speed and pursuit
balance. The possible navigation stall needs reproduction.

The historical game is SkiFree; this project is FreeSki. Chris Pirih's
[official history and downloads](https://ski.ihoc.net/) distinguish the 1991
original from his 2005 Windows release and fixes. This audit uses creator history
and dated first-person accounts, not modern clones as authoritative physics.
We did not measure an original executable's movement or AI. Exact original speed,
acceleration, turning, catch rules and obstacle penalties remain unverified.

## What the original offered, and what players remember

The strongest recurring memory is an overwhelmingly fast yeti. A 2018
[PC Gamer retrospective](https://www.pcgamer.com/remembering-skifree-and-the-yeti-that-still-haunts-our-dreams/)
describes its appearance around 2,000 m, failed childhood escape attempts and the
later discovery that F enables faster skiing. The author reports escaping with
that mode and encountering a second yeti after surviving the first. This supports
severe chase pressure with an escape mechanism; it does not establish an
unconditionally impossible catch or an exact pursuer speed ratio.

Ordinary skiing was not uniformly remembered as fast or hard. A 2002
[GameFAQs review](https://gamefaqs.gamespot.com/pc/578119-skifree/reviews/35600)
calls normal Slalom relatively easy and slow without fast motion, while Tree
Slalom adds obstacle pressure. It also describes Freestyle flips and style scoring.
Its claim that the yeti is unavoidable is a player's impression, contradicted by
accounts of fast-mode escapes. That distinction matters when tuning nostalgia.

A 2022 [player review](https://gamefaqs.gamespot.com/pc/578119-skifree/reviews/173053)
also describes F as a speed toggle and increased crashing amid obstacles.
Players in a 2023 [Reddit discussion](https://www.reddit.com/r/90s/comments/14gmome/skifree_1991_spent_way_too_much_time_playing_this/)
recall repeated deaths and express surprise at learning about F. These are selected
anecdotes, not a representative survey. They suggest that an undiscovered control
contributed to the game's apparently impossible reputation.

A [2011 retrospective](https://whatculture.com/gaming/forgotten-gems-of-gaming-skifree)
also describes other skiers, snowboarders, dogs and slowing moguls. Those add
variety and disruption beyond our static trees/rocks/ramps. Conversely, timed
boosts and life counts found in remakes should not automatically become historical
requirements: a [2013 remake discussion](https://news.ycombinator.com/item?id=5247965)
explicitly identifies limited boost and five lives as that author's additions.

## Why our game can be forgiving

These are source observations at fc9a45a and tuning hypotheses, not newly measured
human outcomes. Practice is deliberately gentle; its experience must not stand in
for untested Slalom difficulty. Tyler separately confirmed a Free Ski chase run.

| Current implementation | Likely difficulty effect |
| --- | --- |
| Player cap 50 m/s, approximately 9.7 s buildup | Straight descent may take too long to feel urgent despite a 180 km/h HUD reading. |
| Fixed forward view of 84.48 m | At top speed there is about 1.69 s of downhill lookahead. Perceived speed depends on this view as well as the speed number. |
| Yeti cap 56 m/s; player 50 m/s | Only a 12% maximum-speed advantage before losses to terrain and turning. |
| Yeti turns at 1.05 rad/s; player at 1.6 rad/s | Committed carving can produce a substantial advantage. Turning also drains yeti speed. |
| Terrain contact retains only 20% of yeti speed | Repeated contacts can destroy sustained chase pressure. |
| Yeti reselects among local heading probes each tick | Wider probes fixed one deadlock, but there is no persistent route/progress memory to ensure reliable detours. |
| Close protected recovery requests zero yeti speed | Some apparent stopping is intentional protection, rather than a navigation defect. |
| Endless terrain reserves a 16 m clear route; hazard centers stay within ±30.5 m on a ±40 m slope | Generous route clearance and persistent outer corridors may allow low-risk escape strategies. |
| One pursuer, no fast-mode control | No extra player speed decision or later escalation of pursuit pressure. |

Sources: [tuning](TUNING.md), [pursuit](../src/chase.rs),
[endless terrain](../src/endless.rs). The prior 33-seed check's longest unprotected
stationary spell was 2.33 seconds; it found no permanent stall in that sample.
That is historical bounded evidence, not proof that the current concern is false.

## Recommended order — proposals, not implemented changes

1. **Make pursuit reliable.** Capture stationary time outside protection and
   reproduce blocked encounters. If needed, use bounded progress tracking and
   stable detours so the yeti keeps advancing around terrain. Preserve physical
   collisions, readable catches and safe crash recovery.
2. **Tune speed and pursuit together.** Compare the existing 50 m/s baseline with
   candidate 60 and 65 m/s runs while preserving steering response. These are our
   experiments, not claimed original values. Forward lookahead would fall to
   roughly 1.41 and 1.30 seconds. Retune pursuer pressure in the same experiment:
   increasing player speed beyond the existing 56 m/s yeti cap alone makes escape
   easier. Target difficult, earned escapes; do not script inevitable death.
3. **Remove trivial terrain strategies if playtests confirm them.** Test sustained
   edge skiing and wide safe corridors before narrowing or varying routes. Keep
   a traversable path and recovery clearance; retain a gentle Practice course.
4. **Consider a visible F-key fast/tuck mode.** This is the clearest missing
   historical mechanic worth trying: greater escape speed at greater obstacle
   risk. Document it rather than relying on a hidden trick. Decide its exact
   behavior after the speed comparison; a timed recharge is a new design choice.
5. **Escalate only if still necessary.** A warned second pursuer could eventually
   restore pressure, but first make one pursuer formidable and dependable.

Keep the three-crash rule and approved controls/art/sound. Moving NPCs and trick
combos are expressly outside the current [issue scope](REQUIREMENTS.md)
([issue #14](https://github.com/tcballard/omarchy-retro-arcade/issues/14)); moguls and
Freestyle are optional later work, not prerequisites for correcting pursuit.
Moving our warning from 1,000 m toward the reported original 2,000 m would delay
pressure, not solve the current complaint.

## Evidence required for the next iteration

Use identical seeds and normal production Session input to compare straight
descent, carving, edge skiing and crash recovery. Record warning-to-catch time,
separation, contacts and unprotected stationary spells. Check that new avoidance
actually escapes the obstruction without bypassing collisions.

Then play a short block of Free Ski runs with both controls and record whether
speed feels urgent, obstacles remain readable, catches feel deserved and escape
requires sustained skill. Include failures, not just the best run. Test Practice
and all five Slalom courses before accepting any shared speed change; recalibrate
medals if necessary. Version behavior/terrain and handle existing saves and
records explicitly. Current proposals have no new runtime test evidence.
