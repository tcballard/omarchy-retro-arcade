# Implementation decisions for Tom's review

Recorded 12 September 2026 UTC.

| Decision | Reason and trade-off |
| --- | --- |
| Standalone Qt 6 / QML application | Own launcher entry and Wayland window; no dependency on the Omarchy plugin shell. |
| Python core through PySide6, instead of the initially suggested C++ | The environment can run Qt's Python bindings. This enabled direct native UI testing and simpler rules/persistence work. On Arch it uses the packaged Qt runtime; Python adds modest startup/runtime overhead. |
| Klondike first, draw-one/draw-three | Completes the agreed first-release game. FreeCell and Spider remain separate future game modes, not partly implemented menu entries. |
| Unchanged official Omarchy logo | Exact SVG geometry and official green fill from the requested brand page. The surrounding card colours follow the theme. |
| Original card faces | Clear pips and geometric J/Q/K faces; no dependency on Microsoft's artwork or commercial decks. Face-card illustrations can be replaced later. |
| Woven, Diamond and Minimal backs | Three original vector patterns with one consistent central logo. |
| Holographic finish is optional and hover-driven | Provides a working effect for the team to preview with new artwork, without continuous idle animation. Reduced motion is static. |
| Imported SVG/PNG replaces the whole back | Gives artists full control. Import copies into app storage. Theme artwork uses the app-defined `solitaire/back.svg` or `.png` convention. |
| Theme palette changes do not touch game state | The adapter polls file metadata every 1.5 seconds and detects directory replacement. No shell hook modification, IPC dependency or commands run from theme files. |
| Readable card faces remain ivory and red/black | Theme matching must not obscure suits or gameplay. UI foreground/accent contrast is corrected against the table. |
| Unlimited recycle and full move undo | Relaxed desktop solitaire. The save parser has a 32 MB byte limit for defensive reads; normal games are far smaller. |
| Random deals, no solvability claim | Fast and honest. Hints rank legal moves that reveal cards and advance foundations; they are not a solver. |
| Conservative automatic finish | Offered only when all cards are visible and stock/waste are empty. Each foundation move is separately undoable; Stop is available. |
| Explicit score rules | Common solitaire-style scoring, documented in Help; no claim of exact Windows-version parity. |
| Statistics count a deal once | Restarts and undo don't inflate starts or wins. Best time is based on active play time. |
| Timer pauses when inactive or in a dialog | Fits short desktop breaks. No timed competitive modes or background time accumulation. |
| Local atomic JSON and one writer | Simple inspectable saves, resume plus undo, lock against concurrent writers, and recovery copies for unreadable saves. |
| Arch package built from the checkout | Reviewable local package now; CI emits a package artifact. Tagged builds prepare a draft release. No AUR or marketplace submission assumed. |
| MIT for new code; logo excluded | Brand rights remain as stated on the official page. The app is labelled as a community application. |

## Evidence used for theme integration

Inspected `bin/omarchy-theme-set` and `themes/tokyo-night/colors.toml` in the
available omacom/omarchy checkout at commit
`5b91db503c904bbfc5f34bdaaa9c708814958f3d`. That implementation stages a next
theme, replaces `~/.local/state/omarchy/current/theme`, and stores its name at
`current/theme.name`. The app also supports the earlier config path.

## Still needs product acceptance

- Real Omarchy desktop experience: launcher, tiling, Wayland fractional scaling,
  actual theme switching, and screen-reader behaviour.
- Art direction sign-off and the team's special deck artwork.
- Whether to add a solver/winnable deals or additional solitaire variants later.
- Public release and distribution channel choice after the native desktop check.

## Rust Arcade milestone — supersedes the Python/Qt implementation

- Rust is now the implementation language, following Tom's Arcade default.
  egui/eframe 0.31.1 matches the collection's Rust desktop approach and supports
  Wayland, X11 and AccessKit. The pinned Cargo.lock is committed.
- Replace the Python runtime and QML, retaining the previous implementation in
  Git history. Python is used only by the optional development screenshot script.
- Preserve the version-1 game and session formats. Golden fixtures were produced
  by the original Python engine before removal. A local MT19937 compatibility
  implementation preserves Python's integer-seeded shuffle and restart behavior.
- Preserve the original save once before migration. Use an OS file lock plus a
  Qt-compatible lock reservation so the old app cannot concurrently write.
- Use original mirrored court portraits with rank-specific headwear, flowers and
  sceptres. Horizontal rank/suit indexes remain visible under overlapping cards.
- Use short 180ms deal, flip, move and return transitions. Reduced motion bypasses
  transitions and celebration; idle rendering wakes at most twice a second,
  apart from window/input activity and explicitly hovered holographic backs.
- Compress hidden-card spacing first in short windows. Keep visible indexes at
  a readable size and scroll only when a long arrangement exceeds the viewport.
- Rasterize imported SVG at a bounded 500 × 700 size, including validation, to
  avoid unbounded allocations from intrinsic SVG dimensions. Static shapes only.
- Keep the previous relaxed scoring rules. Exact historical Windows scoring,
  Vegas, a solver, sounds and additional solitaire variants are outside this milestone.
- Build a 0.2.0 Arch package. CI tests native rendering plus package installation,
  launch and removal. A real Omarchy acceptance session remains a release gate.
- This PR includes the root-PKGBUILD correction from PR #1, since migrating the
  package replaces that Python packaging path. Neither PR is merged automatically.

## Approved engraved deck

Replace the first geometric deck with the approved engraved direction. Pattern
IDs stay 0/1/2/3: Tilework, Engraved, Foil, Special. Existing saves keep their
selected slot and custom artwork. Horizontal serif indexes remain visible under
tableau overlaps; lower indexes and court portraits rotate 180 degrees. The
three generated rank portraits are shared across suits, with explicit suit
medallions and indexes. Backs are authored SVG with theme-derived inks and the
unchanged official mark. Mipmap filtering prevents fine artwork aliasing.
All assets are embedded in the executable; there is no new runtime dependency.

## Number-card design pass

Use DejaVu Serif Bold at a consistent 78-unit ink height on the 500-unit card.
Normalize glyph bearings and derive suit placement from visible rank width, with
24 units of clear separation. Ten retains its natural proportions. Corner insets
are 32 × 28 units, within the tableau exposure. The pip field spans y=190–510,
with symmetric side columns at x=155/345 and larger low-rank pips; nine and ten
use a denser four-row layout. The lower index is still an exact 180-degree copy.

## Arcade save-before-leave

Expose a fallible final save to the Arcade host. Stop automatic completion and
clear selection before saving. A rejected original remains protected. The host
retains the app on failure and skips a second exit write after save or discard.
