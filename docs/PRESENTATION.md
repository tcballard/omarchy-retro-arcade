# Arcade presentation

The collection uses one visual language: machined charcoal surfaces, warm ivory type, brass framing and restrained Omarchy accent light. Circuit's approved illustrated table is the reference for the level of craft. Each game keeps its own identity and readable playfield.

## Opening the collection

All nine games are visible in the selector. A large preview, game name, genre and Play action describe the current selection. Arrow keys select; Enter launches. Tab reaches the native controls. Returning to Arcade retains the selected game. Circuit retains its explicit confirmation before ending a table.

## Treatments

| Game | Presentation work |
| --- | --- |
| Circuit Pinball | Approved table remains intact; common navigation; screenshot capture waits for an actual table frame. |
| Stack | Recessed board surround, bevelled blocks, clearer reserve/next labels and a composed mode-selection screen. |
| Snake | Machined boundary, contact shadows, body highlights and jewel-like food; head and food remain distinct by shape. |
| Bubble | Framed playfield and dimensional glass bubbles; all six non-colour symbols remain legible. |
| Blast | Bevelled metal walls and crates, framed arena and retained original characters. Enter starts a match and advances rounds. |
| Scram | Layered maze edges and engraved wall surfaces; original characters and collision geometry remain intact. |
| Chess | Board surround, subtle square grain and contact shadows beneath the approved pieces. |
| Solitaire | Fine cloth texture and card shadows; approved decks, artwork choices and animation remain intact. |
| Invaders | Framed instrument display and collection materials around the approved pixel artwork. |

`shared/presentation` centralizes reusable materials and control geometry. The cabinet texture is decoded once per egui context, and static detail introduces no new animation. Light themes use an engraved light surface instead of dark artwork behind light-theme text. Existing motion, audio and persistence preferences remain authoritative.

## Verification

The integrated revision passes 197 workspace tests, strict Clippy and formatting. The release executable builds, the three Pinball engine/theme tests pass, and staged installation contains one executable, one desktop entry, per-game licenses and the new artwork provenance.

Native screenshots, keyboard switching and clean Arch install/upgrade verification run in `.github/workflows/arcade.yml`. `scripts/polish-renders.py` captures every game and the shelf in dark, light, compact and 200% layouts, using real XTest input. It contains no alternative gameplay or renderer.

All 40 screenshots from the [integrated CI run](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34689873161) were visually reviewed. The native checks and Arch package installation/reinstall passed. Review corrected unavailable font glyphs in navigation, widened the opening Pinball preview to include its scoreboard, and refined keyboard selection after mouse focus. The [final correction build](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34690454743) passed the full native job: formatting, strict Clippy, all 197 tests, release build, engine tests, nine-game switching, saves, Stack/Snake/Blast interaction checks and captures. It supplies the updated screenshots below. The final Arch job also passed the complete package build, installed-game checks and save-preserving reinstall. All three integration PRs are merged into `main`.

A local sandbox limitation prevents opening a display socket in this session, so the rendered review uses GitHub's `polish-review` artifact. Automated Linux/X11 evidence does not replace hands-on Omarchy/Wayland playtesting or audible-device acceptance.

The separate optional leaderboard service remains disabled for public use. No hosting, account requirement or second launcher is introduced by this presentation change.

## Actual application captures

Captured from the native executable at revision `4e7da43023191548a410c50e09e7947e19974408`. No compositing or replacement UI is used. The full dark/light/compact/200% matrix is in the final build's `polish-review` artifact.

![The opening collection](polish/shelf.png)

| Game | Native window |
| --- | --- |
| Circuit Pinball | [Open screenshot](polish/pinball.png) |
| Solitaire | [Open screenshot](polish/solitaire.png) |
| Scram | [Open screenshot](polish/scram.png) |
| Invaders | [Open screenshot](polish/invaders.png) |
| Chess | [Open screenshot](polish/chess.png) |
| Stack | [Open screenshot](polish/stack.png) |
| Snake | [Open screenshot](polish/snake.png) |
| Bubble | [Open screenshot](polish/bubble.png) |
| Blast | [Open screenshot](polish/blast.png) |

[Light collection](polish/shelf-light.png) · [Compact collection](polish/shelf-compact.png)

## FreeSki complete local modes

FreeSki joins the thirteen-game source build with original snowy terrain, skier,
gates and a shaggy yeti inside the existing cabinet. It offers Practice,
endless Free Ski with optional pursuit, and the five-course Slalom Cup. Original
synthesized cues accompany visible outcomes; mute and reduced effects are saved.

The linked committed captures below the historical report document the initial
practice build. The completion report identifies later dark/light/compact/200%
captures, live pursuit art, course results and exact suspended-run restoration.
[Actual app captures and acceptance notes](../games/freeski/docs/VERIFICATION.md).
[Acceptance and review status](../games/freeski/docs/NEXT.md).
