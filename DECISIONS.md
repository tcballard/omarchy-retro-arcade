# Integration decisions

- One repository, native window, desktop identity, Arch package and release version. Games are ordinary source subdirectories, not submodules or downloaded plugins.
- Four Rust/egui games are library dependencies of the Arcade executable. Preserve approved rendering, gameplay and existing storage paths. Acquire each game's original session lock before opening it, flush on leaving, and drop it on returning to the shelf.
- Circuit retains its C++ upstream engine. A private bundled worker renders through SDL software into the Arcade window. This avoids X11 window embedding and works with the same frontend on Wayland. The worker has no visible window or desktop entry. Its framed local pipes carry pixels and input, never network traffic.
- Existing per-game save directories and artwork choices remain authoritative. No bulk move or conversion risks existing saves. Circuit retains high scores/settings; like its source version it does not restore unfinished games.
- Shared navigation is Ctrl+H / Back to Arcade. Existing game shortcuts remain available. Leaving Circuit explicitly confirms ending the current table.
- Source repositories and open PRs remain intact until the consolidation is accepted. Imported Git history and source hashes make every migration traceable.

## Stack and optional community leaderboards

- Stack is a Rust library game inside the same eframe window, desktop entry and package. Both modes work offline; its state uses a new `omarchy-stack` directory without changing existing games' save locations.
- Stack preserves unreadable, invalid and future-version saves in place and disables session writes until the game is reopened after manual recovery. Unsaved play has a persistent notice. It never automatically replaces an earlier recovery file or claims that a failed backup succeeded; valid saves keep the existing schema and atomic-write behavior.
- Gameplay uses deterministic 60 Hz ticks and documented Stack-specific symmetric rotation kicks. The service links this same engine with UI dependencies disabled. Rules are versioned independently from the app release.
- Shared HTTP transport is in `shared/leaderboard`; the separately deployable SQLite service is in `services/leaderboard`. No public URL is bundled. Explicit end-of-run sharing, pseudonymous credentials, bounded replay verification and private retry storage are required before results become public.
- The initial service is deliberately single-process and intended for a small community. Deployment behind the supplied HTTPS proxy, backups and operational checks must be verified before activating public sharing. Hosting is a separate approval gate, estimated at $11/month before tax in HOSTING.md.
- Snake is a Rust/egui library under `games/snake`, appended to the existing shelf and rendered in the same window. Shelf indexing derives from `Game::ALL.len()` to accommodate concurrent additions without replacing artwork or reordering the approved five entries.
- Snake's `snake-v1` engine and replay adapter build without desktop features. The concurrent Stack-only leaderboard is an explicit dependency; Snake performs no network activity and does not ship a second service. See `games/snake/docs/LEADERBOARD-HANDOFF.md`.
- Snake uses existing theme loading and atomic writes, an independent schema-versioned save directory, and separate records/session files. No other game's save or settings schema changes.
- Bubble is a Rust/egui library under `games/bubble`, loaded by the existing Arcade game trait. It adds no executable, desktop identity or installer. Six shelf entries now derive keyboard wraparound and counters from `Game::ALL`.
- Bubble uses analytic swept-circle shots against a staggered 10/9-column hex grid. A shot's computed polyline drives both the live animation and limited guide; rule resolution occurs once after flight, so frame rate cannot change attachment. Pressure moves the ceiling rather than changing grid parity.
- Bubble levels and their deterministic magazines are authored JSON. Removed colours fall back to the lowest remaining colour. The saved reference solutions include a quarter-degree aiming margin and are replayed through the production engine.
- Current Arcade audio and settings are per-game, not a central settings service. Bubble follows the existing `Ctrl+M` sound convention and owns/reaps its optional `paplay` process on pause and exit. It reuses the existing Omarchy theme loader and stores versioned, atomically replaced progress at `omarchy-retro-arcade/bubble.json` beneath XDG state. Corrupt/future saves remain untouched. Attempts restart when reopened; unlocked levels and personal bests survive.
- Blast is a Rust library in `games/blast`, hosted by the existing `ArcadeGame` lifecycle in the same window. No binary, desktop entry, package or release is added. Its singleton protection is the Arcade session lock.
- Blast reuses the collection's Omarchy theme loader, atomic private storage helper, Ctrl+, settings / Ctrl+M audio conventions, and shared shelf navigation. The current consolidation retains per-game preferences; Blast stores only its own `blast.json` in the Arcade state directory and does not migrate legacy game saves. There is no existing controller service to bind to.
- Arena simulation is integer 60 Hz. Movement intentions use snapshot occupancy: swaps and contested destinations both fail. Explosion chains use a common tile snapshot so a destroyed crate shields every ray in that wave. Elimination is simultaneous after hazard resolution. Newly revealed items stay uncollectable while their tile burns.
- Bots forecast the same hazard rules, including future crate destruction, chain reactions and closing walls. They search traversable positions over time and place a useful bomb only with a predicted escape. Forecasts cannot anticipate future bombs or future movement by other participants. All actions use the ordinary player rules.
- Blast artwork is original egui geometry and its audio consists of original synthesized PCM cues. No reference-game artwork, character designs or sounds are imported. The owned playback child is terminated and reaped on pause and game exit.

## Cohesive Arcade presentation (12 September 2026)

- This feature branch integrates the existing Stack, Snake, Bubble and Blast implementations with the five consolidated games. It preserves each engine, original artwork, saves, rules and optional leaderboard boundaries. No service is deployed or activated.
- The visual thesis is a crafted retro-futurist cabinet: charcoal metal, warm ivory lettering, brass edges and restrained Omarchy accent lighting. The opening screen uses a large art preview and an explicit nine-game selector. Keyboard selection and returning to the same game are part of that design.
- `shared/presentation` owns reusable material rendering, bevels, glass, control geometry and the cached cabinet texture. Gameplay does not depend on these rendering functions; Stack and Snake's UI-free engine features stay UI-free.
- The new cabinet image is decorative only. Boards, collision boundaries, pieces, projectiles, scores and buttons remain real native drawing and interactive widgets. Theme-aware light surfaces use a quiet engraved treatment instead of placing light-theme text over a dark bitmap.
- The approved Pinball table, Invaders sprites, Chess pieces and Solitaire deck remain authoritative. Their artwork is not replaced. A new asset and its provenance live under `shared/presentation/assets`.
- Pinball screenshot capture now waits for a rendered frame rather than capturing its loading spinner. Automated X11 rendering and input evidence remain distinct from hands-on Omarchy/Wayland acceptance.

## Mouse-friendly operation (issue #6)

- Keep Omarchy as the target desktop. Shared navigation and applicable game actions must be clickable; keyboard controls remain first-class. This adds no GNOME support commitment or requirement to make every game entirely mouse-playable.
- Preserve existing mouse gameplay in Chess, Solitaire and Bubble. Invaders adds pointer steering at the engine's existing movement speed and held-primary-button firing, confined to the owned playfield. Keyboard steering takes priority until fresh pointer input.
- Pinball keeps its original engine and menus. The host explicitly gates worker input while confirming return to Arcade, including the closing frame, and releases outstanding pointer presses on blocking/focus loss. Coordinates alone do not establish ownership of an input event.
- The per-game audit and desktop acceptance checklist live in `docs/MOUSE-SUPPORT.md`. No saves, preferences schema, artwork or packaging contract changes.
## 2048 integration (12 September 2026)

- Accepted direction from Tom Ballard: fold avibarit/2048 into Arcade, with credit to the original author. Tom reports permission from the author. This is the integration proposal and scope record for the change.
- Preserve Avi Barit's full original Git history in the validated `games/2048/upstream-history.bundle`, alongside an unchanged source snapshot under `games/2048/upstream/`. Direct Git push is unavailable in this environment, so connector publication retains the original commits in the bundle rather than attaching their identities as PR ancestry. Port the JavaScript gameplay to Rust/egui in `games/2048/src/`, using the ordinary `ArcadeGame` lifecycle, one window, desktop identity and package. The archived Tauri wrapper is excluded from the workspace and is not built or installed.
- Credit Avi Barit as the source implementation author and Gabriele Cirulli as the original 2048 creator in the game, Help, shared About, README and installed notices. Preserve upstream MIT declarations and document the absence of a separate upstream licence file. New Arcade integration remains GPL-3.0-or-later.
- Keep classic merging, spawning, score, win/continue and the current source's 20-move undo. Preserve the latest rapid-move fix's intent: animations have no independent mutable board and completion renders authoritative state. Reduced motion is optional. No audio is added to the silent source game.
- Use a versioned `omarchy-retro-arcade/2048.json` save, private atomic writes and bounded loading. Save all moves, undo, best score and preferences; invalid/future files remain untouched and play continues with an explicit unsaved-state message. No existing game data or standalone Tauri/browser localStorage is migrated or removed.
- Append 2048 to the shelf. Existing ordering and commands remain intact. Mouse direction buttons and board dragging supplement arrows/WASD. Keep headless X11 results distinct from hands-on Omarchy/Wayland acceptance.

### Classic pinball keyboard controls

Z and slash alias A/D with shared held-key state. Release the logical action only when its final physical alias releases and clear held state on blur. Forward period and the supported legacy function keys without rewriting saved engine bindings.

## Shatter (issue #7)

- Append an original Rust brick breaker to the existing shelf and ArcadeGame lifecycle. No executable, desktop identity, network service or new package dependency is introduced. Engine and persistence also build without desktop features.
- `shatter-v1` uses an 800 × 600 arena and 120 Hz deterministic ticks. Exact swept circle/rectangle face and corner contacts include relative paddle movement. Stable brick order resolves shared contacts. Sixteen impacts per tick bound pathological contact loops; ordinary maximum-speed travel is only 4.34 units per tick. A frame delay above 250 ms explicitly pauses, with fresh-input resume.
- Pointer and keyboard movement both move at at most 650 units/s, avoiding an abrupt paddle jump on source switching. Opposite keys cancel. Pointer movement only owns control after a new in-arena movement. Pauses, overlays and host input blocking clear pending launch and require all gameplay inputs to be released.
- Original layouts are fixed Rust data with names and teaching notes. A deterministic paddle controller clears all 20 through ordinary production inputs. A separate flood-fill checks permanent steel cannot seal destructible pockets. This is repeatable engine evidence, not a human feel assessment.
- Campaign state and progression share a new versioned private atomic `omarchy-retro-arcade/shatter.json`; practice is a separate in-memory run and only updates separate per-level bests. Resumption always pauses. Invalid, incompatible and future saves disable writes until an explicit archive-and-reset action succeeds. No other game's data is migrated.
- Existing Omarchy palette and shared cabinet materials frame native geometry. New cues reuse Bubble's owned/reaped PCM player. All assets and layouts are original and documented. Sound and reduced effects remain per-game preferences, matching the existing app.
- Native screenshot/input checks and Arch install/upgrade gates are included in CI. Hands-on Omarchy/Wayland acceptance remains required before calling issue #7 complete.

### Connected Circuit boundaries and traversable routes

Use connected two-sided capsule chains, closed obstacle bodies, explicit ground
and raised layers, and a ground underpass beneath the high ramp arch. Preserve
full-ball-width playable routes and intentional drains. Only the lower mouth
enters the raised tube; its upper portal is outgoing-only. Close both low tube
supports without spanning the underpass. Diagnostics reject illegal interiors,
rail penetration, trapping and wrong entry provenance; they never teleport or
rescue production balls. Keep approved art and upstream physics unchanged.
### Reliable spring charging and contact

Align the ground ball and contact head to the visible coil, animate existing coil pixels from real charge, and retain a 0.75-second one-shot release window through rapid re-presses. The upstream plunger default remains zero for imported resources. Charge text reflects real engine state; the artwork file is unchanged.
### Passive scoring-target response

Stand-up targets and side modules score through the upstream wall response with zero powered boost. Powered bumpers and slings retain their impulses. Contact tracing and real-engine shots distinguish scoring events from energy injection. Geometry is unchanged here.
### Circuit bridge rendering

Prefer the SDL offscreen video driver with accelerated rendering and retain software fallback. Create the bridge window at its final dimensions before its renderer. Copy the rectangular table texture directly in ImGui draw order, avoiding software textured-triangle work while retaining overlays and unchanged artwork. This follows the tiled-quad visibility workaround with a direct-copy path.
### Responsive pinball bridge

Debounce bounded logical surface sizes and resize the SDL render target without restarting the worker. Tall layouts use the full playfield plus a lower HUD, and the host paints its full panel. Pointer coordinates follow the displayed frame; upstream mouse ownership and dialog gating are preserved.
### Circuit elapsed simulation time

Retain up to100ms elapsed time and advance it in bounded120Hz substeps so render/transport stalls do not discard ordinary simulation time. Preserve classic-resource timing. A slow-consumer bridge test compares elapsed wall and engine time.
### Circuit nudge and tilt feedback

Render bounded displacement from active upstream nudge flags. Pause/focus loss
releases held nudges and centres the board. Display DANGER/TILT in the custom HUD;
retain upstream flipper/scoring penalties and next-ball recovery. Track held
input separately from the0.4-second physical pulse: a rested meter warns around
0.875s and tilts around1.75s. Classic-resource behavior remains unchanged.

### Contributor integration

Combine #21–#28 on current main while retaining contributor commits. CircuitGeometry replaces the alternate #19 layout and duplicate launcher constants. Retain #19's boundary/depth regression intent and bumper-cap occlusion using the shared geometry. Use direct plate copies with portrait cropping; one status priority for both layouts (pause, game over, tilt/danger, charge, notice). Exercise normal fixed-time launches without a contact-release test shim. Wire native classic-control, resizing, geometry and nudge regressions into CI. #29 save protection and #30 build-job limits land independently.

### Bounded Pinball worker shutdown

Close the command channel before joining the writer, even if its bounded queue
cannot accept quit. Retain the two-second child termination fallback. Exercise
full-queue, blocked-pipe and exited-child cases with the production writer and
worker destructor under an outer subprocess deadline. This changes transport
teardown only; upstream physics and frame/input protocols remain intact.

### Shatter polish

Keep the v1 physics and save schema stable during presentation polish. Use explicit
serve/pause states, persistent pause explanations, named practice choices and
save retry feedback. Verify menu clicks through real egui events and include all
eleven entries in the native mouse harness. Separate CI screenshots and automated
input from hands-on Omarchy feel acceptance.

### Rejected Blast and Snake saves

Block Blast writes after a rejected load, and block Snake writes when either
recovery archive fails. Conservatively keep Snake's records and session together
until reopening successfully loads or archives both. Ordinary write errors remain
retryable. Repairing files does not silently enable writes in an already-open
fallback game. Preserve existing paths and schemas; test restart, persistence and
exit through real app methods without a native display.

### Player package contents

Keep the pinned Stockfish engine/network bundled for offline computer Chess.
Install only the explicit player-files.tsv payload plus Stockfish and its licence.
Retain player help, component licences and textual asset provenance; screenshots,
design references, test evidence and architecture documents remain in source.
Publish debug symbols separately and install only the player package in Arch CI.
Check the extracted payload against the allowlist and report component sizes.
This changes packaging only, not artwork, gameplay, save paths or recovery policy.

### Circuit score-name editing

Forward committed egui text and paste separately from gameplay keys, using bounded
hex-encoded UTF-8 so whitespace cannot inject line-oriented bridge commands.
Synchronize standalone modifier changes, and retain physical keys for text-editing
shortcuts without changing the classic flipper aliases. Only deliver text when
ImGui requests it. Existing score rows edit a temporary name buffer; OK or Enter
commits names and the existing verification checksum immediately, while Cancel
discards changes. Preserve scores, ordering, save paths and the 31-byte name format.
Cover the real ImGui dialog and native host typing/save/restart in regression tests.

## Tanks engine foundation (issue #8)

- Start Tanks as a dependency-free, UI-independent Rust workspace library under
  `games/tanks`. This engine milestone does not add a shelf placeholder or change
  the native application, existing games, approved assets, saves or package payload.
- Use a 120 Hz simulation, piecewise linear heightfield and swept point-projectile
  contacts. Resolve blast damage from one snapshot, then crater/settle both tanks
  and award the result once. Explicit Ready and RoundOver states let the later
  frontend implement safe handover and draw acknowledgement.
- Preview rules, support geometry, damage rounding and numeric tuning are recorded
  in `games/tanks/README.md`. Values remain provisional until recorded playtesting.
  Cloned simulation state is not yet a disk persistence contract. AI, native UI,
  storage, audio and Omarchy acceptance remain subsequent slices of issue #8.

### Tanks playable preview

- Append Tanks as the twelfth shelf entry using the existing ArcadeGame lifecycle,
  theme loader and cabinet presentation. Native geometry, labelled numerical aim,
  explicit fire and turn handover support mouse and keyboard in one window.
- Easy/Normal AI incrementally evaluates ordinary engine shots. Normal also tries
  limited repositioning; both compare limited weapons and penalise self-damage.
  Work has a fixed per-call tick/candidate budget. Pausing discards search; resuming
  reconstructs it from the saved visible match and dedicated AI seed, preserving
  the eventual choice without accessing future terrain randomness.
- Versioned tanks.json uses bounded validated reads and shared atomic private
  writes, with exact projectile/RNG/trace state. Rejected saves disable writes until
  explicit unique archival succeeds. Match records and preferences are logically
  separate from active match state. Reopening always pauses.
- This is a silent playable preview. Effects, original audio, visual refinement
  and hands-on benchmark comparison remain open; reduced-effects preference is
  reserved for upcoming animation. Headless checks do not establish Omarchy feel.

## Tanks impact and control polish

- Keep damage resolution in the deterministic engine. `tick_event` returns an
  immutable pre-impact snapshot and actual blast/fall damage; the frontend saves
  a separate 108-tick presentation. Pause, shelf, close and reopen retain exact
  settling progress. No commands or AI advance until presentation completes.
- Reduced effects uses final positions with static feedback, preserving the same
  rules and turn delay. Effects use stable visual noise, never the engine RNG.
- Original bounded synthesized PCM cues use an owned, reaped paplay process.
  Mute, pause, focus loss and shelf exit stop playback. Missing or failed audio
  is nonfatal and reported in Settings. No new package dependency is introduced
  (the Arch package already includes libpulse).
- Fresh installs choose Solo Easy, Solo Normal or Local. Optional saved fields
  preserve old preview matches. Held aiming is time-based; explicit Fire and
  release-to-rearm prevent handover inputs from becoming accidental shots.
- Add original layered terrain, track details, aiming arcs, a wind flag, recoil,
  flashes, weapon-specific impacts and damage labels. Native X11 renders are
  inspected at dark/light, compact and 200%; this does not establish Omarchy
  Wayland acceptance or competitive balance against the gameplay benchmark.
