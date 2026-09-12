# 2048 for Omarchy Arcade

Native adaptation of [Avi Barit's 2048](https://github.com/avibarit/2048),
with permission reported by Tom Ballard. Original 2048 by
[Gabriele Cirulli](https://github.com/gabrielecirulli/2048).
Full [credits, licence evidence and source provenance](THIRD_PARTY.md).

Choose **2048** from the collection or launch `omarchy-retro-arcade --game 2048`.
It runs in Arcade's existing window and package, using the desktop theme.
No Tauri/WebKit dependency, browser, separate launcher or network service is added.

## Rules and controls

- Classic 4×4 board, starting with two tiles; each spawn is 2 (90%) or 4 (10%)
  in a uniformly selected empty cell.
- Equal tiles merge once per move. Score increases by the merged values.
  Unchanged moves neither spawn nor score.
- Reach 2048, choose Keep going, and continue. A full board with no merges ends
  the game. Best score survives undo and starting another game.
- Arrows or WASD, clickable direction buttons, or drag across the board to move.
- Undo with Ctrl+Z, U, Z or the button, up to 20 moves, including after game over.
- Esc pauses/resumes; F1 opens Help & credits. All actions are clickable.
- Ctrl+H returns to Arcade, F11 toggles fullscreen and Ctrl+Q quits.
- Help includes a reduced-motion preference. The upstream game has no audio;
  this adaptation is silent too.

The current code's undo behaviour is authoritative; the older upstream spec
predates that feature. Rendering uses the latest committed board after each
animation, preserving the intent of upstream's rapid-move regression fix.

## Saves

The board, score, best, continued-after-win flag, up to 20 undo snapshots and
reduced-motion preference save after each change and on leaving/closing, to
`$XDG_STATE_HOME/omarchy-retro-arcade/2048.json` (default
`~/.local/state/omarchy-retro-arcade/2048.json`). Writes use a private temporary
file and atomic replacement. Invalid, oversized and future-version files remain
untouched; the UI explains that play is temporary until the file is repaired or
moved aside. Write errors remain visible and saving is retried on later changes.

The standalone browser/Tauri game's localStorage remains untouched. It is not
automatically imported into Arcade. Existing Arcade game saves are unchanged.
Unlike upstream, undo history is retained across reopening. A new game confirms
discarding the current board. RNG state is not restored by undo, matching the
upstream implementation's ability to generate a different tile on replay.

## Development checks

```sh
cargo test --locked -p omarchy-2048
node games/2048/upstream/test-game-logic.js
node games/2048/verify-upstream.cjs
```

Run commands from the workspace root. Node is only used for the developer
comparison; it is not an application/runtime dependency. The comparison covers
3,524 moves through both real engines, including scores and animation metadata.
See [verification](docs/VERIFICATION.md) for native acceptance evidence and gaps.

The `upstream/` snapshot is excluded from the Cargo workspace and is preserved
byte-for-byte. Do not format, rebuild or patch that snapshot to change the port.

The complete original Git history is in `upstream-history.bundle`. To inspect
Avi’s original commits, run `git clone games/2048/upstream-history.bundle /tmp/2048-upstream`
from the workspace root. The published PR preserves those commits in this bundle,
rather than attaching them to the repository ancestry.
