# 2048

Exact classic clone of [2048](https://play2048.co/) built with pure vanilla HTML, CSS, and JavaScript — plus a native Linux desktop app.

## Install (one line)

**Omarchy / Arch** (builds the native app, then installs to `~/.local`):

```bash
sudo pacman -S --needed git rust nodejs npm webkit2gtk-4.1 gtk3 libsoup3 base-devel && git clone https://github.com/avibarit/2048.git && cd 2048 && ./build-native.sh && ./install.sh
```

**Debian / Ubuntu** (prebuilt `.deb` from [releases](https://github.com/avibarit/2048/releases)):

```bash
curl -fsSLO https://github.com/avibarit/2048/releases/download/v1.0.0/2048_1.0.0_amd64.deb && sudo dpkg -i 2048_1.0.0_amd64.deb; sudo apt-get install -f -y
```

**Fedora / RHEL** (prebuilt `.rpm` from [releases](https://github.com/avibarit/2048/releases)):

```bash
curl -fsSLO https://github.com/avibarit/2048/releases/download/v1.0.0/2048-1.0.0-1.x86_64.rpm && sudo dnf install -y ./2048-1.0.0-1.x86_64.rpm
```

Then launch with `2048` or from your app launcher.

## How to play

Open `index.html` directly in any modern browser (double-click or File → Open).

- Use **arrow keys** (or **WASD**) to slide tiles
- On touch devices, **swipe** on the board
- Merge identical tiles to create larger powers of two
- Reach **2048** to win (you can keep playing after)
- Fill the board with no more moves = Game Over
- **Best** score is stored in `localStorage`

## Files

| File | Purpose |
|------|---------|
| `index.html` | Page structure |
| `styles.css` | Classic colors, layout, animations |
| `game.js` | Pure logic + renderer + controls |
| `test-game-logic.js` | Node unit tests for core rules |
| `2048-spec.md` | Original requirements |

## Running tests

```bash
node test-game-logic.js
```

Tests cover:

- Sliding and merging (including merge-once-per-move)
- Score calculation and merge position metadata
- Random tile spawn (2/4 with 90/10)
- Win and lose detection
- No tile added / score change on invalid moves

## Behavior (classic rules)

- Starts with exactly **2** random tiles
- Only adds a tile after a **successful** move
- `4 2 2 2` left → `4 4 2` (merge once)
- Win overlay on 2048; **Keep going** continues play
- Game over when the board is full and no adjacent equals
- Full keyboard, touch swipe, New Game, score + best

No frameworks, no build step, zero runtime dependencies.

## Native desktop app (Omarchy / Arch Linux)

A native Tauri + WebKitGTK wrapper lives in `src-tauri/`. It reuses the same
`index.html` / `styles.css` / `game.js` — no rewrite.

Requirements (Omarchy/Arch):

```bash
sudo pacman -S rust cargo nodejs npm webkit2gtk-4.1 gtk3 libsoup3
```

Build + install to `~/.local` (no sudo):

```bash
npm install
npm run build   # or: ./build-native.sh
./install.sh
```

Then launch with `2048` or from your app launcher (wofi/rofi).
The installer places:

- `~/.local/bin/2048`
- `~/.local/share/applications/2048.desktop`
- `~/.local/share/icons/2048.png`

Packaging: a `PKGBUILD` is included for `makepkg` / AUR-style installs
(`makepkg -si`), which installs to `/usr/bin/2048`.
