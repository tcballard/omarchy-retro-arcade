# Shatter

An original brick breaker inside Omarchy Arcade. Launch with the shelf or
`omarchy-retro-arcade --game shatter`. It works offline in the existing app window.

Move with the pointer, Left/Right or A/D. Click or Space serves; hold to fire when
Laser is active. Escape pauses/resumes; Ctrl+, opens Settings; Ctrl+M mutes;
F1 opens Help; Ctrl+H returns to Arcade; Ctrl+Q quits. Menus are clickable and
keyboard-focusable. Releasing held gameplay inputs is required after resuming.
Pointer position maps through the letterboxed arena, with a 650-unit/s movement
limit to avoid teleporting on input-source changes. Leaving the window pauses.

Campaign starts with three lives and 20 fixed levels. Solid bricks score 100;
inset armour takes two hits and scores 200; crossed steel cannot be destroyed.
Clear all destructible bricks for 1,000 points. Next Level resets temporary
powers, preserving lives. Level 20 ends the campaign. Practice uses unlocked
levels without changing the suspended campaign or campaign records.

E expands the paddle for 15 seconds, M splits up to three balls (250 points at
the cap), L enables twin lasers for 12 seconds. Repeat timed pickups refresh;
effects can coexist. Drops use a saved xorshift state and 15% eligibility roll.
A life is lost only when the final ball falls. A same-tick clear wins over loss.

The fixed arena is 800 × 600, simulated at 120 Hz. Ball radius is 6, starting
speed 300, damaging contacts add 5 up to 520. Paddle impacts steer between 10°
and 65° from vertical; serves use 15°. Bricks use 62 × 27 spacing, 56 × 20 faces,
starting at (28, 58). `src/levels.rs` contains the patterns and challenge notes.
At 12 seconds without damage, a cue accompanies redirection toward a visible
destructible target at unchanged speed. If none has a clear swept path, a
fixed inward/upward heading exits the corridor before the next attempt.

Delays above 250 ms pause rather than dropping collision time. Timed effects
advance only in active simulation. Reduced effects removes trails/fragments;
there is no camera shake. Original synthesized audio can be muted.

Campaign checkpoints occur every 15 seconds, on pause, shelf return and close.
`$XDG_STATE_HOME/omarchy-retro-arcade/shatter.json` (default `~/.local/state/...`)
contains the versioned exact state, unlocks, records and preferences. Reopen is
paused. Unsupported/corrupt files remain untouched; Recovery explicitly archives
the original to a unique private file before reset. Practice is not resumable.

[Verification evidence and remaining desktop checks](docs/VERIFICATION.md).
[Asset provenance](THIRD_PARTY.md).
