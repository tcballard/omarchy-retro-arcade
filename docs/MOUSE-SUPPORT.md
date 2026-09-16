# Mouse support

Omarchy Arcade supports mouse-friendly navigation alongside its keyboard controls. This work targets Omarchy; it does not add a GNOME support commitment.

## Shared controls

Click a title in the collection, then **Play**, or double-click the title to launch it. **Arcade** returns to the collection and keeps the selected title. **Full screen** toggles fullscreen on both the collection and game screens. Arrow selection, Enter, Ctrl+H, Ctrl+Q and F11 remain available.

Each game keeps its own settings and help. There is no new shared settings store or save migration.

## Game audit

| Game | Mouse gameplay | Clickable actions / instructions |
|---|---|---|
| Chess | Click source and destination squares, or drag pieces; click promotion choice | New game, Game, Settings, Help, board flip, rematch. No timed pause action applies. |
| Solitaire | Click or drag cards; double-click/right-click sends a card to a foundation | New game/restart, undo, hint, settings, Help and Deck. |
| Bubble | Move over the playfield to aim; click to shoot | Pause/resume, restart, sound, How to play, level selection and next level. |
| Invaders | Move over the field to steer at normal ship speed; hold left click to fire | Game (new/pause), Settings, Help; clickable Resume and Play again on the field. Keyboard steering takes over immediately; move the mouse to steer again. |
| Circuit Pinball | Table gameplay remains keyboard-controlled | The embedded Game, Settings and Help menus accept clicks; scrolling is forwarded over the embedded image. The Arcade return confirmation blocks worker input, including its closing click. |
| Scram | Directional gameplay remains keyboard-controlled | Game (new/pause), Settings, Help, Play again. |
| Stack | Falling-block gameplay remains keyboard-controlled | Mode selection, new/resume saved run, pause/resume, restart, preferences/control bindings, return. |
| Snake | Directional gameplay remains keyboard-controlled | Speed/start, pause/resume, restart, Settings/control bindings, return. |
| Blast | Movement/bombs remain keyboard-controlled, including local two-player | Match/arena selection, start, Settings/controls, pause/resume, restart, next round, return. |

| 2048 | Click direction buttons or drag across the board | New game, undo, pause/resume, Help, preferences and continue after reaching 2048. |

The shared actions are mouse-accessible; this is not a promise that every game is entirely mouse-playable. Rebinding a keyboard control still requires pressing the replacement key. Future games should provide suitable mouse gameplay and clickable common actions from the outset.

## Changes for issue #6

The game audit found most common actions already clickable. The patch adds Invaders steering/fire, collection mouse instructions and pointer feedback, clickable fullscreen controls, and ownership-aware Pinball pointer forwarding. Invaders pauses simulation while a Game menu is open without preventing its Pause/resume item from resuming play after the menu closes.

Pointer input is confined to the Invaders playfield and disabled while paused, unfocused, in menus/dialogs or after game over. A press started outside the field cannot fire by dragging into it. Mouse steering follows the existing movement speed and collision rules; it does not teleport the ship. Pinball releases an outstanding mouse press when host input is blocked or focus is lost, so the worker cannot retain a stuck button.

## Acceptance on Omarchy

Automated egui input tests and X11 native checks are separate from an actual Omarchy/Wayland playtest. Before closing #6, verify on Omarchy:

- All ten games can be selected, launched and left using the mouse; Pinball can both cancel and confirm returning.
- Applicable settings/help, pause/resume, restart and new-game actions work with clicks.
- Chess, Solitaire, Bubble, Invaders and 2048 accept the documented pointer gameplay.
- Opening/closing menus and dialogs cannot also play a move, fire a shot or activate a worker menu underneath.
- Keyboard controls still work immediately after mouse use; leaving the field, releasing a held click and losing focus stop pointer firing.
- Collection, game controls and confirmation dialogs remain usable in light/dark themes, compact windows and 200% scale.

### Minesweeper

Left-click reveals; right-click flags; clicking a revealed number or middle-clicking chords. Arrow keys move the focus, Space reveals, F flags and C chords. Esc pauses. Menus hide the board and suspend the timer.
