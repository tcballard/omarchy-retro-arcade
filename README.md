# Omarchy Arcade

**Ten games. One native app. One more go.**

A collection of classic games for Omarchy. Play pinball, cards, puzzles and arcade games in one native window, with offline play, local saves and an interface that follows your desktop theme.

**[Try the preview](#install)** · [See every game](docs/PRESENTATION.md#actual-application-captures)

![Omarchy Arcade's opening collection, with a full Pinball preview and all nine games in the selector](docs/polish/shelf.png)

Circuit Pinball · Solitaire · Scram · Invaders · Chess · Stack · Snake · Bubble · Blast · 2048

**Development preview for x86_64 Omarchy.** The nine-game build passes native Linux and Arch installation checks. [Verification and remaining desktop playtesting](docs/VERIFICATION.md).

2048 is included when building this source revision. The older preview download below
contains nine games and does **not** include 2048.

## Install

1. [Download the verified nine-game preview](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/34690454743/artifacts/10296419019) from GitHub Actions. Sign in to GitHub if prompted.
2. Extract the ZIP and open a terminal in the extracted folder.
3. Install the package:

```sh
sudo pacman -U ./omarchy-retro-arcade-*.pkg.tar.zst
```

Open **Omarchy Arcade** from your app launcher. Choose a game with the arrow keys and press `Enter`. `Ctrl+H` returns to the collection; `Ctrl+Q` quits.

The package includes all nine games and a bundled Stockfish engine for Chess. It replaces conflicting standalone game packages while retaining their existing save files and settings. The app works offline and needs no account.

This is a tested development build from 12 September 2026, not a stable release or an official Omarchy package. The download supports **x86_64**; an ARM package is not available yet. GitHub's artifact expires on 11 December 2026. After that, use a newer successful build from [Actions](https://github.com/tcballard/omarchy-retro-arcade/actions/workflows/arcade.yml) or [build from source](#build).

## Build

On Arch, install Rust 1.98+, CMake, SDL2, SDL2_image, SDL2_mixer and the native graphics dependencies listed in `packaging/PKGBUILD`.

```sh
scripts/build.sh
./target/release/omarchy-retro-arcade
```

For computer Chess in a source build, provide Stockfish through `OMARCHY_CHESS_ENGINE`. The Arch package builds and bundles the pinned engine automatically:

```sh
packaging/build-arch.sh
```

The package builder requires a clean committed checkout and does not install anything.

## Contribute

PRs, issues and playtesting are encouraged. [Report a bug, suggest an improvement or pitch a game](https://github.com/tcballard/omarchy-retro-arcade/issues/new/choose), or read the [contribution guide](CONTRIBUTING.md) to get started.

## Source layout

- `arcade/`: the Rust app, collection shelf and local Pinball transport.
- `games/`: ten ordinary game directories, preserving all imported Git history.
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
