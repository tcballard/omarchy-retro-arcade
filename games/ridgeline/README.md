# Ridgeline

An original tower-defence campaign inside Omarchy Arcade. Launch it from the
shelf or with `omarchy-retro-arcade --game ridgeline`. It plays offline in the
existing app window. "Ridgeline" is a provisional name (issue #11).

Enemies follow fixed roads, and aircraft follow marked flight lanes, towards the
brass gates. Each enemy that gets through damages the base once. The base starts
with 20 health; at zero the defence falls. Hold all 20 waves on a map to win.
There are ten authored maps, each with Normal and Hard wave tables.

## Controls

| Action | Mouse | Keyboard |
| --- | --- | --- |
| Choose a tower | Click it in the Build list | 1 Cannon, 2 Mortar, 3 Flak, 4 Cryo |
| Move the cursor | Move over the board | Arrow keys |
| Place | Click open terrace | Enter |
| Cancel placement | Right-click or **Cancel placement** | Escape |
| Inspect a tower | Click it | Enter on it |
| Upgrade / sell | Inspector buttons | U, or Tab to the buttons |
| Start the next wave | **Start wave N** | Space (only while waiting) |
| Pause / resume | **Pause** / **Resume** | Escape (after cancelling placement) or P |
| Speed 1× / 2× | **Speed** | F |

Maps, Restart, Settings (Ctrl+,), Help (F1), sound (Ctrl+M) and **Back to
Arcade** (Ctrl+H) are in the toolbar. Ctrl+Q quits. Every panel is clickable
and reachable with Tab. The pointer and keyboard share one cursor; a click in
the Build list or toolbar never places or selects anything on the board.
Leaving the window pauses a running wave. Resuming is always an explicit action.

## Rules

- Build only on open terrace: never on roads or rock. Rejected placements and
  cancellations spend nothing. Building, upgrading and selling work between
  waves, during a wave and while paused.
- Wave 1 waits for you to start it, and so does every later wave. The panel
  shows the next wave's units, health, armour, base damage and whether they fly.
  Nothing moves between waves or while paused.
- A wave ends when every scheduled unit has spawned and each one has been
  destroyed or has exited. Exits are resolved before the result, so base health
  reaching zero is a defeat even on the final wave.
- Kills pay a fixed bounty once, however many shots land in the same tick.
  Clearing a wave pays a fixed award (25 on Normal, 20 on Hard). Units that
  escape pay nothing. Credits never regenerate while waiting.
- Selling refunds floor(70%) of everything spent on that tower.
- Armour subtracts a fixed amount from every hit, with a minimum of 1 damage.
- Cryo slows use the strongest active effect. Each Cryo tier refreshes its own
  timer. Slows never stack and never stop a unit.
- Towers target the eligible unit with the least remaining route distance,
  breaking ties by spawn order. Only Flak hits aircraft; the others hit only
  ground units. Mortars cannot fire inside their minimum range.
- Direct shots never retarget. If their target has died or escaped, they fizzle.
  Mortar shells commit to an impact point that leads the target. They land and
  splash there even if the target is gone, hitting ground units within the
  splash radius at impact.
- Upgrades change a tower's stats immediately but keep its remaining reload.
  Shots already in flight keep their original damage, including after a sale.
- 2× runs twice as many ordinary 60 Hz ticks per second. It never doubles
  damage or skips a tick.
- If a frame is delayed by up to a second, the game runs at most a quarter
  second of ticks and drops the rest, so the defence slows briefly instead of
  jumping ahead. Longer gaps, such as a suspend, pause the game.

Tower, enemy, map and wave data, the Hard table and the balance evidence are in
[docs/BALANCE.md](docs/BALANCE.md).

## Campaign and saves

Winning a map on Normal unlocks the next map and that map's Hard table. Medals
are recorded separately for each difficulty: **Clear** for a victory and
**Perfect** for a victory at full base health. Any unlocked map can be
replayed. Starting a different map while a defence is unfinished asks for
confirmation first. Unlocks, medals and records survive restarts and replays.

`$XDG_STATE_HOME/omarchy-retro-arcade/ridgeline.json` (default
`~/.local/state/...`) is a versioned file written with private atomic
replacement. It holds campaign progress, preferences and the exact active
battle: towers, reloads, units, route progress, slow timers, in-flight shots,
credits, health and wave schedule position. The combat rules use no
randomness, so there is no random state to store. The game saves at every
command, wave start, wave end, pause, focus loss, shelf exit and close, and
every 15 seconds of play. A saved wave always reopens paused.

Unreadable, damaged, incompatible and future-format files are never
overwritten. The game explains the problem and writes nothing until you choose
**Archive and start fresh**, which first copies the original to a unique
`ridgeline-recovery-*.json` beside it.

## Development

```sh
cargo test -p omarchy-ridgeline                         # engine, storage and UI
cargo test -p omarchy-ridgeline --no-default-features   # UI-free engine only
cargo run -p omarchy-ridgeline --release --no-default-features --example strategy-evidence
```

`strategy-evidence` plays every map and difficulty with the reference strategy
through the production engine. With a directory argument it rewrites the
winning replays in `tests/replays/`. `examples/fixture.rs` writes resumable
saves for native checks. See [verification](docs/VERIFICATION.md) and
[asset provenance](THIRD_PARTY.md).
