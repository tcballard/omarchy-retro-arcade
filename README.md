# Omarchy Arcade

[![Built for Omarchy](https://raw.githubusercontent.com/tcballard/omarchy-badges/85f859029e236e784e7b05ada6dbe73506d07a91/badges/v1/built-for-omarchy.svg)](https://github.com/tcballard/omarchy-badges)

<a href="https://github.com/tcballard/omarchy-badges"><img src="https://raw.githubusercontent.com/tcballard/omarchy-badges/75975e5b5bf75e7ede3764bcd2950046f7abfe2c/badges/v1/omarchy-app.svg" height="20" alt="Omarchy App community badge"></a>

**Twelve games. One native app. One more go.**

A collection of classic games for Omarchy. Play pinball, cards, puzzles and arcade games in one native window, with offline play, local saves and an interface that follows your desktop theme.

**[Try the preview](#install)** · [See every game](docs/PRESENTATION.md#actual-application-captures)

![Omarchy Arcade's opening collection, with a full Pinball preview and all nine games in the selector](docs/polish/shelf.png)

Circuit Pinball · Solitaire · Scram · Invaders · Chess · Stack · Snake · Bubble · Blast · 2048 · Shatter · Tanks

**v0.2.0 release preparation for x86_64 Omarchy.** Twelve games, including
2048, Shatter and Tanks. [Release notes and readiness](docs/releases/v0.2.0.md).
The versioned package is being prepared; it has not been published yet.

## Install

1. [Download the verified twelve-game development build](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34886679784/artifacts/10365483280) from GitHub Actions. Sign in to GitHub if prompted.
2. Extract the ZIP and open a terminal in the extracted folder.
3. Install the package:

```sh
sudo pacman -U ./omarchy-retro-arcade-[0-9]*.pkg.tar.zst
```

Open **Omarchy Arcade** from your app launcher. Click a game and **Play**, or double-click its title. You can also select with the arrow keys and press `Enter`. Click **Arcade** (or press `Ctrl+H`) to return; **Full screen** and `F11` toggle fullscreen, and `Ctrl+Q` quits. See the [mouse controls and per-game input guide](docs/MOUSE-SUPPORT.md).

The package includes all twelve games and a bundled Stockfish engine for Chess. It replaces conflicting standalone game packages while retaining their existing save files and settings. The app works offline and needs no account.

That download was built from `8d1d74d` on 14 September 2026 and still reports
version 0.1.0. It is the tested baseline, not the forthcoming v0.2.0 package.
It supports **x86_64**; no ARM package or live Omarchy/Wayland acceptance is claimed.
GitHub Actions downloads expire; [current builds](https://github.com/tcballard/omarchy-retro-arcade/actions/workflows/arcade.yml)
remain available separately from versioned releases.

Release preparation produces a `release-v0.2.0-x86_64` artifact with the player
package, corresponding source, release notes, build identity and SHA-256 checksums.
Install only the player package; debug symbols are optional and separate.
See [player package contents](packaging/PLAYER.md).

## Build

On Arch, install CMake, SDL2, SDL2_image, SDL2_mixer and the native graphics dependencies listed in `packaging/PKGBUILD`. `rust-toolchain.toml` pins the Rust version, which rustup installs on first build.

```sh
scripts/build.sh
./target/release/omarchy-retro-arcade
```

The build uses every core. Set `ARCADE_BUILD_JOBS` to limit it.

### Install a source build

`scripts/install.sh` joins a `DESTDIR` and a `PREFIX`, so a system install reads:

```sh
sudo scripts/install.sh / /usr
```

It installs no Chess engine. For a computer opponent, set `OMARCHY_CHESS_ENGINE` to a Stockfish executable, or put one where the app looks:

```sh
sudo install -Dm755 "$(command -v stockfish)" /usr/libexec/omarchy-retro-arcade/stockfish
```

### Build an Arch package

The package bundles the pinned Stockfish automatically:

```sh
packaging/build-arch.sh
```

The package builder requires a clean committed checkout and does not install anything.

## Contribute

PRs, issues and playtesting are encouraged. [Report a bug, suggest an improvement or pitch a game](https://github.com/tcballard/omarchy-retro-arcade/issues/new/choose), or read the [contribution guide](CONTRIBUTING.md) to get started.

## Source layout

- `arcade/`: the Rust app, collection shelf and local Pinball transport.
- `games/`: twelve ordinary game directories, preserving all imported Git history.
- `packaging/`: one Arch package, icon and desktop entry.
- `scripts/`: shared build, staging and verification entry points.
- `shared/presentation/`: shared cabinet materials, control styling and artwork.
- `shared/leaderboard/`: optional background HTTP transport.
- `services/leaderboard/`: separately deployable replay-validation service; no public endpoint is bundled.

Nine Rust games draw directly into the shared window. Pinball retains the upstream C++ physics engine in a private worker whose rendering appears in that same window, including on Wayland. No browser, X11 child-window embedding or separate game launcher is used.

Existing save paths remain authoritative. Pinball preserves high scores and settings, but does not resume unfinished tables. The other games save when returning to Arcade.

Bubble adds 20 authored bubble-shooting puzzles, saved level progress and personal bests. [Controls, verified routes and native screenshots](games/bubble/README.md).

See [migration provenance](docs/MIGRATION.md), [integration decisions](DECISIONS.md), and each game's licence and artwork notices. The combined application is distributed under GPL-3.0-or-later; permissively licensed components retain their notices. This is a community project.

Stack includes offline Marathon and 40-line Sprint, local records and exact resumable runs. See [Stack rules](games/stack/docs/RULES.md), [verification](games/stack/docs/VERIFICATION.md) and the [leaderboard hosting proposal](services/leaderboard/HOSTING.md). Public sharing awaits deployment approval.

2048 is adapted from [Avi Barit (avibarit)](https://github.com/avibarit/2048), with permission reported by Tom Ballard. Original 2048 by [Gabriele Cirulli](https://github.com/gabrielecirulli/2048). [Controls, saves and credits](games/2048/README.md).

Shatter is an original brick breaker: 20 authored levels, mouse/keyboard play, three power-ups, campaign saves and unlocked-level practice. [Controls and verification](games/shatter/README.md). Hands-on Omarchy acceptance is pending.

Tanks is an artillery preview with solo Easy/Normal AI, local two-player matches, destructible hills and resumable matches. [Controls and current limitations](games/tanks/HELP.md). Includes original sound cues, animated impacts and settling, aiming aids and reduced effects. Hands-on Omarchy acceptance remains pending; [polish verification](games/tanks/VERIFICATION.md).
