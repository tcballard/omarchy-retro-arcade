# Omarchy Space Cadet

**Omarchy Circuit** is a self-contained pinball table powered by the SpaceCadetPinball engine. It uses upstream ball physics, collision handling, flippers, bumpers, plunger and drain, with newly authored geometry, illustrated artwork and scoring rules.

No Windows game files are needed for default play. This is an original table, not a reconstruction of Space Cadet's table or missions.

![Actual authored table running in the upstream engine](docs/upstream-circuit.png)

## Install and play

Download an Arch package from a successful [Linux build](https://github.com/tcballard/omarchy-spacecadet/actions/workflows/linux.yml), unzip it, and install:

    sudo pacman -U /path/to/omarchy-spacecadet-package.pkg.tar.zst

Or run makepkg -si from the checkout's packaging directory. Open Omarchy Space Cadet from the app menu.

A/D operate the flippers. Hold Space, then release to launch. P pauses, Escape pauses, F2 starts a new game, and F11 toggles fullscreen. Controls are configurable in Settings. This table is single-player with three balls. Bumpers score 100; twelve hits complete a 2,500-point circuit. Targets score 250; completing both four-target banks awards 5,000. A launch carried through the right orbit scores 1,000 and reaching the raised ramp’s upper sensor scores 1,500. Slingshots score 25. Both outlanes lose the ball. Tapping or holding the nudge keys builds toward a tilt. A ball that comes to rest in open play is kicked free after about six seconds.

Settings contains Appearance and Sound. Follow Omarchy uses the active desktop palette for green glass, lamps and UI accents, retaining the artwork’s ivory, chrome and amber materials. Mute controls the synthesized collision effects. Circuit has no background music; that option is disabled. The official logo stays unchanged.

## Local data and updates

Update with pacman -U using the newer package. Circuit settings and high scores are separate from classic settings, normally under ~/.local/share/omarchy-spacecadet/circuit/ (respecting XDG_DATA_HOME).

Circuit uses the upstream game's session handling: high scores and settings persist, but an unfinished game does not resume after closing. Previous experimental saves remain untouched.

## Other modes

    omarchy-spacecadet --classic --data-dir /path/to/original-resources
    omarchy-spacecadet --experimental

Classic mode loads your original Space Cadet or Full Tilt resources. Experimental mode preserves the earlier separate physics prototype and its saves. Neither is used for default play.

## Implementation and verification

Authored component records are built in memory using the upstream DatFile representation. CircuitView renders the approved illustrated board with live ball, flippers, lamps and dot-matrix displays. TRamp handles a separate raised collision surface; the physical and visual coordinate systems share the same mapping. SDL2_image loads the packaged PNG. No downloaded original assets are used. A small separate scoring controller handles Circuit events; Space Cadet's mission controller is retained for classic mode.

The default-engine integration test runs 180 simulated seconds through the actual executable, requires a finished three-ball game, checks finite bounded ball state, and uses no external DAT. CI builds and installs the Arch package and repeats that test against the installed engine. Separate directed shots test ramp, target, orbit and drain collisions. Experimental save-upgrade tests remain separately labelled.

See [table design](docs/ORIGINAL_ENGINE.md), [decisions](DECISIONS.md) and [Arcade standard](docs/ARCADE_STANDARD.md). Real Omarchy desktop checks and hands-on tuning remain. Game code is MIT; official branding retains its owner's rights.

Version tags matching the package version build GitHub prereleases with packages and checksums. A development PR is not a release.
