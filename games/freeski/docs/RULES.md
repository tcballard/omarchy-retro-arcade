# FreeSki rules v3

Practice is an authored 1,200-metre run. Free Ski continues across seeded terrain,
with optional creature pursuit and separate distance records. Slalom has five
courses, sequential unlocks, ordered gates, penalties and medals. A third crash
ends an attempt; a creature catch ends a chase. Practice and Slalom also end at
their finish flags. Player movement uses rules 3; save schema is 4.

## Movement and camera

World coordinates are metres, x across the slope and +y downhill. A UI-independent
60 Hz engine owns position, speed, heading, flight and recovery. Steering approaches
a desired heading at a bounded rate. Holding keyboard steering turns up to ±90°; releasing retains the current heading;
mouse steering retains its latest heading target until the next intentional input.
Braking can bring the skier to rest. Turning sideways slows descent. At the slope
edge, outward lateral movement is constrained; downhill movement still follows
heading. Ordinary movement never jumps to the pointer or travels uphill.

Speed builds gradually to 60 m/s on straight snow. F toggles fast tuck during any
active run, raising the cap to 90 m/s without changing steering authority. The
toggle is also a visible button. There is no cooldown or stamina bar. Leaving fast
mode sheds excess speed gradually; braking still works in either mode.
Fast mode visibly folds the skier into a low tuck with narrow skis, close hands
and trailing poles. Braking, jumps and tumbles retain their distinct poses;
reduced effects keeps the tuck visible.
The fixed 96 × 96 m view shows the same terrain at every supported window size.
The skier sits 12% down the view, leaving 84.48 m (about 1.41 seconds at normal top
speed or 0.94 seconds in fast mode) of downhill look-ahead. The HUD is outside the collision playfield. Resize
changes projection only. Rendering does not mutate physics or obstacle locations.
A backlog above 250 ms pauses clearly instead of dropping simulation time.

## Endless Free Ski

Choose Free Ski before starting, or switch from the pause/results screen. Replacing
an unfinished attempt requires confirmation. The mountain continues beyond 1,200 m;
three crashes end the run. Each new mountain gets a seed outside the simulation.
Saving and reopening retains that seed, terrain, momentum, heading and timers.
Practice completions and Free Ski distance records are separate.

The generator reconstructs bounded chunks from the seed and generator version.
Only nearby chunks remain in memory; future terrain is prepared beyond the visible
view before it can appear. Trees, rocks, optional ramp jumps and open stretches
vary by seed, with a connected clear route. Generator 2 narrows and varies that
route and places hazards across both outer edges; a fixed edge line is no longer
a safe route. Edge hazards are separated so recovery retains a clear fallback.
Density rises to a fixed cap. Enable Creature pursuit before starting to add the
chase described below.

## Creature pursuit

The option defaults off. At 1,000 m the creature gives a visible warning lasting
three simulation seconds before seeking a clear spawn behind the skier. A blocked
spawn retries at bounded intervals. Active pursuit never teleports: it accelerates,
turns and collides with terrain in world coordinates. If the forward routes are
blocked, it chooses a bounded, persistent detour and physically turns around
terrain; contact still stops movement rather than passing through obstacles.
For interception it leads the skier's physical motion by up to half a second,
brakes to fit a tight approach and turns at up to 4 rad/s. A nearby clear approach
takes priority over a detour around terrain beyond the target. Its higher ordinary
straight speed creates pressure. Fast tuck can outrun it on
clear snow, while obstacle hits and poorly timed turns risk losing that advantage.
The creature is drawn at its physical position, with a labelled distance marker
when outside the view. The warning remains visible when muted.

Swept relative contact catches the skier even during a jump, ends the run and
records the chase distance once. During tumble and recovery protection, catches
are disabled and the creature stops outside a safe gap. Timers and actor motion
freeze on pause; restoration does not restart warning or pursuit. New mountains
retain the selected chase option. Switching categories cannot transfer records.

## Slalom Cup

Pinecone Path, Long Turns, Split Pines, Needle Run and Summit Cup unlock in order.
A downhill physical crossing between a gate's poles counts the next ordered gate;
passing outside adds five seconds once. Sideways/uphill motion and recovery
relocation do not count. Poles use ordinary crash physics with a 1.2 m collision height. The first valid finish after resolving every gate ends the race.

The timer starts on intentional start/steer and counts simulation ticks, including
recovery but excluding pauses. Final time is raw time plus missed-gate penalties.
Results show these separately. Bronze requires a valid finish; Silver and Gold
use the authored time targets in TUNING.md. Finishing unlocks the next course.
Personal bests, unlocks and preferences survive retries and mode changes. Creature
pursuit is disabled in Slalom. The course chooser disables locked courses.

## Ramps, obstacles and recovery

Practice has three striped ramps; Free Ski generates optional ramps. Both launch
deterministic 1.2-second, 2.5-metre-high jumps. Airborne
steering has 40% authority. The ground shadow remains the collision reference.
Low rocks have 0.65 m collision height; trees remain dangerous at every jump height.
Circular collision footprints use analytic swept intervals, including descent into
an obstacle during the tick. Ramp contact starts flight at its contact time.

One collision spends one of three allowances. After a nonfatal crash, a bounded
search selects nearby clear snow to the side or uphill, with collision-validated edge
positions as fallback. Recovery never increases downhill
records. A 0.7-second tumble stops movement, followed by 1.5 seconds of visible
collision protection. Pause freezes both timers. A fatal collision at or before
the finish takes precedence over finishing; ended runs never restart themselves.

## Controls and lifecycle

| Action | Keyboard | Mouse |
| --- | --- | --- |
| Ready mode / course | P/F/L; 1–5 for unlocked Slalom courses | Mode and course buttons |
| Ready pursuit option | C in Free Ski | Creature pursuit checkbox |
| Start | Enter or a fresh steering key | Start skiing |
| Steer | A/D or Left/Right | Move within the slope, left/right of the skier |
| Fast tuck | F during a running attempt | Fast / FAST ON button |
| Brake | Hold S or Down | Hold right button on slope or Hold to brake |
| Pause/resume | Esc; Enter resumes | Pause / Resume skiing |
| Restart | Tab to the action and activate; Enter confirms replacement | Restart practice slope or New mountain, then Replace run |
| Help | F1 | Help |
| Settings | Ctrl+, | Settings |
| Mute | Ctrl+M | Mute sound in Settings |
| Return to Arcade | Ctrl+H | Shared Arcade button or Back to Arcade |

Space activates focused UI buttons and is not a gameplay brake.

The most recent fresh keyboard steering press or intentional in-field pointer
movement selects the input source. A stationary or out-of-field pointer cannot
steal keyboard control. Focus loss pauses; regaining focus requires deliberate
resume. Overlays consume their input and held gameplay input must be released
before it can control the resumed run. Help/settings return to a paused run.
Reduced effects hides cosmetic tracks, powder, landing puffs and yeti stride.
Tracks are bounded to 240 segments and follow the direction of travel. Powder
uses twelve recycled points; at most one landing puff lasts 24 simulation ticks.
These effects never change collisions, speed or saved progress.
Original synthesized cues mark carving, ramps, crashes, gates, missed gates,
creature warnings, catches and finishes. Ctrl+M or Settings mutes sound. Playback
uses the desktop paplay service, with no audio device required for silent play.
At most one owned process plays a cue; pause, mute and exit stop and reap it.

## Saves and records

`$XDG_STATE_HOME/omarchy-retro-arcade/freeski.json` (or the equivalent
`~/.local/state` path) is independent of all other games. Schema, rules and course
versions are 4, 3 and 1 respectively; new endless mountains use generator 2.
The save contains the active run including fast mode and pursuit navigation state,
separate record/preference fields and an exactly-once result marker. Practice is
static. Free Ski reconstructs terrain from seed, position and generator version.

Validated schema-1/2/3 and rules-1/2 saves migrate and resume paused. Original bytes
are retained in a non-overwriting backup before rewriting. Position, momentum,
flight/recovery, mode, seed, timers, preferences and course unlocks survive. Old
active mountains keep generator 1 until replaced; they never regenerate beneath
the player. Old scores move into retained records visible in Settings. A migrated
attempt is labelled Legacy run and banks into those records until restart or mode
selection starts the new record domain. Fresh attempts use the new rules and
terrain, with normal and fast tuck available within the same run; existing
pursuit-on/off record separation remains. Old Slalom times do not award newly
calibrated medals. Invalid legacy states still fail validation, and failed backup
creation leaves the original untouched.

Native writes reuse the existing private atomic-storage helper. The headless
runner supplies an equivalent temp/write/sync/rename/directory-sync implementation;
both feature builds exercise round-trip, retention and permissions tests. JSON
float round-trip parsing preserves f64 state for replay on the same build/platform.
Bitwise replay across different CPU/libm implementations is not claimed.

Reads are limited to 64 KiB and reject unsupported versions, invalid numeric state
and inconsistent outcomes. A failed load preserves the original and offers either
unsaved play or an explicit archive/reset. Archives never overwrite existing
archives. A failed write pauses play and offers retry; the previous file survives.
Other games' saves, preferences and data paths are unchanged.
