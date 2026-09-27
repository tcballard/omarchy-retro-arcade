# Arcade lifecycle audit

Baseline: `d0a7839394f0e93aca20ca56f8255966e560dd27` (19 September 2026).
This is a source audit; desktop acceptance is recorded separately.

## Shared host boundary

The eframe raw-input hook now filters input **before egui computes widget clicks**.
While unfocused, it removes keyboard, text, pointer and scroll events, clears held
keys/pointer state, and preserves focus/capture notifications and viewport metadata.
Controls held over blur are suppressed until a release, then a fresh press.
Returning focus alone never synthesizes a resume command. A press swallowed during
blur may require releasing and pressing again; this deliberately avoids repeat-driven
movement or accidental activation after returning to Arcade.

This closes a same-frame ordering hole: Stack, Bubble, Blast, Scram, Invaders and
2048 can pause on blur and then process an Escape/P shortcut later in the same
update. Filtering the incoming frame ensures that shortcut cannot undo the pause.
It also protects host Ctrl+H/Ctrl+Q and turn-based game widgets from inactive input.
A post-update event clear would be too late for egui's cached widget interactions.

Host Pinball confirmation retains its existing separate input gate, including the
closing frame. It must accept dialog input while focused, so the raw focus filter
is not used to consume dialog controls. No generic game-modal API is introduced.

## Game contracts inspected

| Game | Focus/pause contract | Exit and ownership |
| --- | --- | --- |
| Chess | Turn-based; background engine analysis may continue. Cancels unfinished drags; no new pause mode. | Cancels engine and persists in on_exit; session lock outlives game. |
| Solitaire | Cancels unfinished drags; timer stops when inactive/dialog open and resumes on active play. | Persists session; legacy lock outlives game. |
| Scram | Active play pauses on blur; explicit resume. | Persists on exit; legacy lock retained through destruction. |
| Invaders | Pauses on focus loss after being focused; simulation also checks focus. | Persists on exit; store owns locking. |
| Circuit Pinball | Blur/modal sends pause and clears held bridge input. | Private worker quit, bounded grace then kill/reap; threads joined. |
| Stack | Pauses, clears input latch and stops audio. | Persists on exit; resumable state remains game-owned. |
| Snake | Running/countdown enters pause; buffered turns/clock handled by game. | Persists on exit; owned audio cleanup. |
| Bubble | Pauses and stops audio; in-flight simulation requires focus. | Saves progress; active attempts restart by existing design. |
| Blast | Pauses, clears pending bombs and accumulator, stops audio. | Preferences saved; existing match policy unchanged. |
| 2048 | Pauses and clears animation on blur. | Saves board/undo state; no audio process. |
| FreeSki | Pauses running session, clears controls and clock, saves. | Flushes resumable state; owned audio cleanup. |
| Shatter | Suspend clears pending launch, controls and clock; stops audio. | Persists resumable state; owned audio cleanup. |
| Tanks | Suspend clears aiming/movement state and AI search; stops audio. | Persists match/presentation state; owned audio cleanup. |
| Minesweeper | Playing board pauses; elapsed clock checks focus and input gate. | Flushes exact board and records; no audio process. |
| Ridgeline | Running wave pauses on blur/input gate; clears pending placement and clock; stops audio. | Persists exact battle, progress and preferences; owned audio worker dropped. |

The host's Active destructor still calls on_exit once and destroys the game before
releasing its save lock. It does not force every game through suspend on exit:
several suspend methods save already, and each on_exit owns its final save contract.
Turn-based games and games that deliberately restart attempts retain their policies.

## Owned sound playback

Chess, Scram and Invaders now own a shared `Playback` worker through their game
object. It accepts one cue while idle and drops overlaps. The worker owns its
short-lived file and `paplay` child, applies the existing two-second playback
limit, and kills/reaps the child before removing the file. Dropping the game
signals cancellation, wakes the worker and joins it before returning; queued
work checks cancellation before starting playback. No sound worker survives
normal game destruction. Idle workers wait on a channel rather than polling.

File writes and process creation stay off the frame path. Shutdown interrupts
playback waiting immediately, rather than waiting for the two-second timeout.
The final join still waits for any in-progress OS file/process syscall; this is
not a hard real-time shutdown guarantee on a stalled filesystem/kernel.

## Remaining work

Pinball now requests shutdown once and polls child exit/thread completion between
frames. Returning home, Ctrl+Q, native window close and screenshot completion
retain the active session until cleanup completes; no replacement game opens in
that interval. A quit request takes priority over a pending return home. The
existing two-second grace remains, followed by killing/reaping only the owned
child. Reader/writer threads are joined only after they finish. Active still calls
on_exit once and releases the game before its lock. A blocking destructor remains
as a forced-teardown fallback; normal close/navigation finish polling first.

Live Omarchy testing of focus, switching, closing and audio remains outstanding.

## Checks

Host tests cover blur plus pause shortcut in one frame, held keyboard/pointer
release/rearm, a real egui Resume button across blur/refocus, and capture preservation.
Chess and Solitaire integration tests drag a legal move, blur, then release and
verify that no move is applied.
The existing native Stack script now holds a movement key across focus loss and
asserts refocus/release leave the saved simulation paused. Existing native Pinball
modal/input tests remain required. See VERIFICATION.md for results and limitations.
