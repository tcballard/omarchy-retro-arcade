# Ridgeline balance reference

All values are integers in `src/data.rs`; this page restates them for review.
One cell is 1,000 world units, simulation runs at 60 ticks per second, and the
board is 20 × 12 cells. The values are **proposed v1 defaults**: they come from
engine evidence, not from human playtesting.

## Towers

Cost is the build price for tier 1 and the upgrade price for tiers 2 and 3.
Range and splash are in cells. Reload is in ticks (60 = one second).

| Tower | Tier | Cost | Damage | Range | Min range | Reload | Shot speed (u/tick) | Splash | Slow |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Cannon (ground) | 1 | 50 | 10 | 2.6 | — | 36 | 320 | — | — |
| | 2 | 45 | 17 | 2.8 | — | 32 | 340 | — | — |
| | 3 | 80 | 30 | 3.1 | — | 28 | 360 | — | — |
| Mortar (ground) | 1 | 90 | 16 | 4.2 | 1.5 | 110 | 110 | 0.90 | — |
| | 2 | 80 | 27 | 4.5 | 1.5 | 100 | 115 | 1.00 | — |
| | 3 | 130 | 44 | 4.9 | 1.5 | 90 | 120 | 1.15 | — |
| Flak (air) | 1 | 70 | 7 | 3.2 | — | 14 | 450 | — | — |
| | 2 | 60 | 12 | 3.5 | — | 12 | 470 | — | — |
| | 3 | 100 | 20 | 3.8 | — | 10 | 490 | — | — |
| Cryo (ground) | 1 | 60 | 3 | 2.3 | — | 45 | 260 | — | 30% for 90 ticks |
| | 2 | 55 | 5 | 2.5 | — | 42 | 270 | — | 40% for 110 ticks |
| | 3 | 85 | 8 | 2.7 | — | 38 | 280 | — | 50% for 130 ticks |

Flight time is ceil(distance ÷ shot speed) ticks, fixed at launch. Sell refund
is floor(70% × credits spent). A tower's reload starts at zero when built, and
all reloads reset to zero when a wave ends.

## Enemies

Base health is multiplied by the wave's health percentage. Speed is in world
units per tick (18 ≈ 1.08 cells per second).

| Enemy | Role | Base health | Speed | Armour | Bounty | Base damage | Spawn spacing |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Crawler | Basic ground | 40 | 18 | 0 | 4 | 1 | 36 |
| Scout | Fast ground | 24 | 34 | 0 | 3 | 1 | 22 |
| Plated | Armoured ground | 70 | 14 | 4 | 7 | 2 | 48 |
| Glider | Aircraft | 36 | 22 | 0 | 5 | 1 | 40 |
| Hauler | Slow heavy ground | 260 | 9 | 2 | 20 | 5 | 110 |

## Waves and difficulty

Each wave is a list of groups. A group spawns its units at that kind's spacing,
and the next group starts 75 ticks after the previous group's last spawn.
Ground groups alternate across the map's ground roads and air groups across its
flight lanes, continuing that rotation from wave to wave.

For wave index *w* (0–19), enemy health percentage is
`base + step × w + floor(w² ÷ 3)`, plus Hard's bonus.

**Hard** is its own documented table, derived from Normal:

- every group has ⌈1.3 × size⌉ units;
- enemy health is +15 percentage points on wave 1, rising by one per wave (+34 on wave 20);
- starting credits are 90% of Normal, rounded down;
- the wave-clear award is 20 instead of 25.

Tower accuracy, damage, prices and every other rule are identical on both
difficulties.

| # | Map | Lesson | Routes | Credits N / H | Health % (wave 1 → 20, Normal) |
| --- | --- | --- | --- | ---: | ---: |
| 1 | First Terrace | One winding road. Cannons near the bends cover the most ground. | 1 ground, 0 air | 200 / 180 | 100 → 410 |
| 2 | Dry Wash | Scouts are quick but fragile. Cover the long straights early. | 1 ground, 0 air | 200 / 180 | 100 → 429 |
| 3 | Hawk Ledge | Gliders ignore the road and only Flak can reach them. | 1 ground, 1 air | 220 / 198 | 105 → 453 |
| 4 | Iron Gate | Armour subtracts from every hit. Heavier shots matter here. | 1 ground, 0 air | 230 / 207 | 105 → 453 |
| 5 | Twin Draws | Two roads meet at the ford. Mortars punish crowds. | 2 ground, 0 air | 240 / 216 | 110 → 477 |
| 6 | Mesa Crossing | Ground and air arrive together. Balance the budget. | 1 ground, 1 air | 240 / 216 | 110 → 496 |
| 7 | Split Rock | The road forks. Every group takes the next branch in turn. | 2 ground, 0 air | 260 / 234 | 115 → 501 |
| 8 | Windward | Two flight lanes. Flak placed where they cross does double duty. | 1 ground, 2 air | 260 / 234 | 115 → 520 |
| 9 | Long Drop | A long switchback gives time to wear down Haulers. | 1 ground, 1 air | 260 / 234 | 120 → 525 |
| 10 | Last Pass | Everything at once: two roads, two flight lanes, every enemy. | 2 ground, 2 air | 300 / 270 | 100 → 543 |

Wave tables (Normal). Letters: c Crawler, s Scout, p Plated, g Glider, h Hauler; the number is the group size.

| Wave | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | c6 | c6 | c8 | c8 | c10 | c10 | c10 | g6 | c12 | c8 |
| 2 | c8 | s6 | g4 | p3 | c12 | g6 | s8 | c10 | h1 | s8 g4 |
| 3 | c10 | c8 s4 | c8 g4 | c8 p3 | c14 s4 | c10 s6 | c14 | g10 | c14 s6 | p3 c8 |
| 4 | c12 | s10 | g8 | p6 | c16 | p4 g6 | p4 c8 | c12 g6 | h2 | h1 g6 |
| 5 | c8 c8 | c10 s6 | c10 g6 | c10 p5 | h1 c10 | c14 g8 | s14 | g14 | p6 c12 | c16 s8 |
| 6 | c14 | s14 | s8 g8 | s8 p6 | c20 | h1 c12 | c20 | s10 g10 | h2 c14 | p8 g8 |
| 7 | c10 s4 | c12 s8 | c12 g8 | c12 p7 | c16 p4 | s12 g8 | p8 s8 | g18 | s16 | h2 c14 |
| 8 | c16 | s16 | g12 | p10 | h2 c12 | p8 c12 | h2 c10 | p6 g12 | h3 p4 | g16 s12 |
| 9 | c12 s6 | c14 s10 | c14 s6 g8 | c12 p8 s6 | c24 s8 | g14 c10 | c24 s10 | g22 | c24 g8 | p14 c16 |
| 10 | c20 | s20 | g16 | p14 | h3 c16 | h2 s10 g8 | p12 c12 | c16 g16 | h4 c12 | h4 g14 |
| 11 | c14 s8 | c16 s12 | c16 g10 | c14 p10 | c28 | c20 p6 | h3 s12 | g26 | p12 s10 | c28 s14 |
| 12 | c18 s6 | s24 | s12 g12 | p12 s10 | p8 c20 | g16 s10 | c28 | h2 g18 | h5 | p16 g14 |
| 13 | c20 s8 | c18 s14 | c18 g12 | c16 p12 | h4 c20 | h3 c16 | p14 c14 | g30 | c28 g12 | h5 c20 |
| 14 | c16 c16 | s28 | g20 | p18 | c32 s10 | p12 g12 | h4 s16 | s16 g20 | h5 p8 | g22 s18 |
| 15 | s14 c14 | c20 s16 | c16 s10 g12 | c16 p14 s8 | h4 p8 c16 | c24 s14 | c30 p10 | g34 | s24 c16 | p18 c20 |
| 16 | c24 s10 | s30 c10 | g22 c10 | p20 c10 | c36 | h4 g14 | s24 c16 | p10 g24 | h6 g12 | h6 g18 |
| 17 | c20 s16 | c22 s20 | c20 g16 | c18 p16 | h5 c24 | p14 c20 | h5 p12 | g38 | p16 c20 | c32 s20 p10 |
| 18 | c28 s12 | s34 | s16 g18 | p22 s12 | c30 p12 | g20 s16 | c34 s16 | h3 g28 | h8 | h7 g22 |
| 19 | c24 s20 | c24 s24 | c22 g20 | c20 p20 | h6 c28 s10 | h5 p12 c16 | h6 p16 c16 | g42 c16 | h6 p14 s16 | p20 g20 c24 |
| 20 | c30 s20 | s40 c20 | g26 c20 s12 | p26 c20 s10 | h8 c36 | h6 g20 c24 s12 | h8 c30 s20 p12 | g48 s20 | h10 c30 g16 | h10 p16 g24 s20 |

## Reference strategy evidence

`examples/strategy-evidence.rs` plays each map with `src/strategy.rs`, a
transparent planner that issues only ordinary player commands between waves.
It reads the same information a player sees: map geometry, tower data and the
next three wave previews. For each spending decision it tries seven budget
styles (how strongly to balance ground against air, and the highest tier to
buy). It rehearses each style for the next three waves on a copy through the
production engine, then keeps the one that loses the least base health. It
never builds mid-wave, never sells, and needs no hidden placement knowledge.

The resulting winning command lists are stored in `tests/replays/`. The test
suite replays them through the production engine, at 1× and at 2× with uneven
frame times. Current output (source build, x86_64 Linux):

| Map | Difficulty | Result | Base health | Towers | Credits left | Combat time |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| 01 First Terrace | Normal | Victory (wave 20) | 20 | 12 | 239 | 6:23 |
| 01 First Terrace | Hard | Victory (wave 20) | 20 | 12 | 279 | 7:37 |
| 02 Dry Wash | Normal | Victory (wave 20) | 20 | 11 | 258 | 7:41 |
| 02 Dry Wash | Hard | Victory (wave 20) | 20 | 13 | 294 | 9:28 |
| 03 Hawk Ledge | Normal | Victory (wave 20) | 20 | 13 | 308 | 6:05 |
| 03 Hawk Ledge | Hard | Victory (wave 20) | 14 | 15 | 374 | 8:16 |
| 04 Iron Gate | Normal | Victory (wave 20) | 20 | 13 | 357 | 9:33 |
| 04 Iron Gate | Hard | Victory (wave 20) | 20 | 15 | 432 | 11:47 |
| 05 Twin Draws | Normal | Victory (wave 20) | 20 | 15 | 360 | 9:22 |
| 05 Twin Draws | Hard | Victory (wave 20) | 20 | 17 | 431 | 11:30 |
| 06 Mesa Crossing | Normal | Victory (wave 20) | 20 | 14 | 419 | 7:44 |
| 06 Mesa Crossing | Hard | Victory (wave 20) | 13 | 14 | 498 | 9:51 |
| 07 Split Rock | Normal | Victory (wave 20) | 20 | 13 | 484 | 6:31 |
| 07 Split Rock | Hard | Victory (wave 20) | 20 | 16 | 604 | 8:29 |
| 08 Windward | Normal | Victory (wave 20) | 20 | 16 | 331 | 9:07 |
| 08 Windward | Hard | Victory (wave 20) | 5 | 18 | 432 | 11:20 |
| 09 Long Drop | Normal | Victory (wave 20) | 11 | 15 | 434 | 13:34 |
| 09 Long Drop | Hard | Victory (wave 20) | 14 | 17 | 575 | 15:20 |
| 10 Last Pass | Normal | Victory (wave 20) | 20 | 17 | 559 | 11:41 |
| 10 Last Pass | Hard | Victory (wave 20) | 10 | 19 | 666 | 14:10 |

Reading this table:

- Every map and difficulty is winnable with ordinary rules and a repeatable
  strategy.
- An undefended campaign loses on every map (`undefended_campaign_is_lost_and_exit_damage_is_exact`).
- The reference planner rehearses its choices in the engine before committing.
  Human players cannot do that exactly, so Perfect results here do **not** mean
  a map is easy for people.
- Hard costs this planner health on Hawk Ledge, Mesa Crossing, Windward, Long
  Drop and Last Pass. It does not on the teaching maps.
- Planner losses concentrate in the first five waves of the later maps. Late
  waves are mainly limited by credits, and large leftover balances at victory
  suggest the late economy is generous.
- Combat time excludes planning between waves, which is unlimited. The issue's
  10–20 minute session target needs human timing.

Tuning history in this pass: the planner originally ignored air threats and
lost 8 of 20 combinations. After adding demand saturation and engine
rehearsal, the remaining failures pointed at early spikes on Last Pass. Its
opening waves, starting credits and health ramp were softened, the Hard bonus
was made to rise with the wave number, and late waves gained the quadratic term.

## Open for human playtesting

- Pacing per map and the 10–20 minute session target.
- Whether Normal is approachable for new players, and whether Hard is too
  forgiving on the first two maps.
- Choice variety: the reference strategy under-uses Cryo and never sells.
  Whether Cryo and upgrade timing feel worthwhile is unmeasured.
- Readability at small window sizes during dense late waves.
