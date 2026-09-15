# FreeSki

An original SkiFree-inspired downhill skiing game inside Omarchy Arcade.

Choose **Practice** for the authored 1,200-metre learning slope, **Free Ski**
for a seeded endless mountain with optional creature pursuit, or **Slalom** for
five courses unlocked in order. All use the same fast carving, brakes, jumps and
three crash allowances. Free Ski keeps separate chase-on/off distance records.
Slalom adds ordered gates, five-second missed-gate penalties and local medals.
Original sound, creature and course artwork are included.

Build with `scripts/build.sh`, choose FreeSki at the end of the Arcade shelf, or run:

```sh
./target/release/omarchy-retro-arcade --game freeski
```

Enter starts. Hold A/D or Left/Right to turn up to 90°; release to keep your heading. Moving the mouse
left/right of the skier selects pointer steering. Hold S, Down, the right
mouse button on the slope, or the visible brake button to slow down. Ramps launch
automatically. Press F during a run (or click Fast) to toggle a faster tuck; it
raises the speed cap from 60 to 90 m/s. Esc pauses/resumes; Ctrl+H returns to Arcade. Settings: Ctrl+,. Ctrl+M mutes sound.

Before starting, P/F/L select Practice/Free Ski/Slalom, C toggles pursuit and 1–5
select unlocked courses. These choices also have clickable controls. Enable pursuit before a Free Ski run; the creature
warns after 1,000 m. Slalom starts with Pinecone Path and unlocks the next course
on completion. Pause/results offer mode changes; confirm before replacing an
unfinished run.

Runs save on pause, close, shelf exit, results and five-second simulation
checkpoints, then reopen paused. Restarting unfinished progress requires confirmation.
Records and preferences survive restarts. Earlier-rule records remain available
in Settings; new runs use the revised difficulty records, with course unlocks retained. Invalid saves remain untouched until an
explicit archive/reset; playing without saving is also available.

- [Rules, controls, save behavior and limitations](docs/RULES.md)
- [What comes next](docs/PLAN.md)
- [System design and ownership](docs/SYSTEM.md)
- [Next playtest](docs/NEXT.md)
- [Milestone 1 implementation and human acceptance](docs/MILESTONE-1.md)
- [Issue #14 requirements snapshot](docs/REQUIREMENTS.md)
- [Tuning and reference runs](docs/TUNING.md)
- [Acceptance and evidence](docs/VERIFICATION.md)
- [Original asset provenance](assets/README.md)

Run `cargo test -p omarchy-freeski --locked` for engine, storage and frontend checks;
add `--no-default-features` for the desktop-independent engine/storage suite.
`scripts/native-freeski.py` exercises the real app under Xvfb. The
`practice-evidence`, `endless-evidence` and `completion-evidence` examples generate suspended-run fixtures
through ordinary production inputs for native reopen checks. The `replay` example
accepts a bounded JSON input schedule through the same Session as the app.

New code and original assets are GPL-3.0-or-later. No SkiFree artwork, sounds,
courses or creature design are bundled. This game has no standalone launcher.
