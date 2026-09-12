# 2048 credits and licence provenance

## Avi Barit's implementation

This game is a native Rust/egui adaptation of **2048 by Avi Barit
([avibarit](https://github.com/avibarit))**, used with the author's permission
as reported by Tom Ballard on 12 September 2026.

- Source: https://github.com/avibarit/2048
- Pinned revision: `3f10becf38aa40a21c1555091f677c272830788a`
- Original source, assets, tests and metadata: `upstream/`, unchanged.
- All four original commits and their authorship are preserved unchanged in
  `upstream-history.bundle`, a complete Git bundle validated with `git bundle verify`.
  The GitHub connector cannot transfer original commit identities into the PR
  ancestry. Recover them with `git clone games/2048/upstream-history.bundle /tmp/2048-upstream`.
- `upstream/src-tauri/Cargo.toml` and `upstream/PKGBUILD` explicitly declare MIT.
  Upstream supplies no separate LICENSE file or copyright notice at this revision.
  We preserve those declarations and reproduce the MIT terms below, without
  representing this document as an original upstream licence file.

The Rust slide/merge operation and motion metadata are adapted from `game.js`.
Tests include the upstream adjacent-128 regression and direct cross-language
comparison. The new UI, persistence and host integration are Arcade contributions
under GPL-3.0-or-later. The upstream source retains its MIT declaration.

## Original 2048

The original **2048 was created by Gabriele Cirulli**:
https://github.com/gabrielecirulli/2048 and https://play2048.co/.
Avi's README and specification identify that game as their reference. We credit
both the original game and the specific implementation adapted here. No assets
or source files are imported directly from Cirulli's repository by this change.

## Assets and distribution

The native game and shelf preview use new geometric drawing, existing Arcade
materials and bundled fonts. `docs/shelf.svg` is an illustrative board, not a
captured playthrough. Upstream icons and web assets remain in the source snapshot;
they are not used as the Arcade application icon or installed as another app.

These credits appear in the game, its Help & credits dialog, the collection's
About dialog, and installed notices under
`/usr/share/licenses/omarchy-retro-arcade/2048/`.

## MIT terms for the upstream component

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
