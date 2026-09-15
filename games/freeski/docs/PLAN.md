# FreeSki delivery plan

The issue #14 playable scope is implemented: practice, seeded endless Free Ski,
optional creature pursuit, five Slalom courses, local medals, sound, keyboard and
mouse controls, and resumable runs. The September 14 difficulty revision adds faster rules-3 movement, an F-key fast
tuck, stronger pursuit and more demanding endless terrain while retaining the
approved steering response. [Verification](VERIFICATION.md)
records checks and separates automated evidence from human acceptance.

| Area | Delivered behavior |
| --- | --- |
| Practice | Authored 1,200 m slope, carving, braking, ramps and three crash allowances |
| Free Ski | Bounded seeded terrain, independent chase-on/off distance records |
| Pursuit | Warning after 1,000 m, physical creature, terrain avoidance, swept catch and protected recovery |
| Slalom Cup | Five sequentially unlocked courses, ordered gates, five-second misses, timed results and medals |
| Session and saves | One production tick path, bounded JSON replay, retained legacy originals, suspended causal state |
| Presentation | Original skier, terrain, gate and creature geometry; original synthesized cues; mute/reduced effects |

The next work is a focused playtest and any fixes it reveals, described in
[NEXT.md](NEXT.md). [Human playtests](PLAYTESTS.md) track acceptance separately from
automation; the [difficulty audit](DIFFICULTY-AUDIT.md) records the research behind this revision. Do not reopen completed feature setup. Medal
targets have production reference runs; those establish feasibility, not human
difficulty. Keep issue #14 open until
its remaining desktop/release acceptance is recorded.

[System design](SYSTEM.md) owns responsibility boundaries, [rules](RULES.md) owns
current behavior, and [tuning](TUNING.md) owns numerical values and observations.
Root CONTRIBUTING.md and CI define combined gates. Actual package installation
and human playtesting must be reported separately from local builds and staging.

Online play, global leaderboards, trick combos, equipment, weather, moving NPC
skiers and course editing remain outside this v1 scope. FreeSki ships inside the
existing Arcade window, desktop entry and package; other games and saves remain.
