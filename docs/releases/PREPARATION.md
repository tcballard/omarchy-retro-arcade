# v0.5.0 release preparation

Ridgeline (issue #11) adds the fifteenth game. The v0.4.0 publication record
is in this file's history at `e6912dfd8ac85463ed532b7a99a709340511eb17`.

This revision bumps all 19 local Cargo packages and the Arch package to 0.5.0.
It also adds the [release-page copy](v0.5.0.md). **There is no verified
candidate yet.** The release is cut from `main` after the Ridgeline branch
merges, as v0.4.0 was.

## Status on 27 September 2026

| Gate | State |
| --- | --- |
| Ridgeline feature branch CI | Green on earlier branch commits: [run 175](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/36315234995) at `574e577` and [run 176](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/36334623319) at `c65daa4`. Native, Arch and Tanks-engine jobs all passed. This evidence does not transfer to later commits. |
| Review and merge to `main` | Not started; no PR open |
| CI on the merged `main` commit, with a release artifact | Pending; this is the candidate |
| Bundle identity and checksum check | Pending; see below |
| Omarchy desktop acceptance | Pending (Tom) |
| Tag and publish | Not authorized |

Local checks for this preparation: the version bump only changes version
fields; `Cargo.lock` agrees on 0.5.0 for every local package. Ridgeline's own
local verification is in
[its verification record](../../games/ridgeline/docs/VERIFICATION.md).

## Steps

1. **Merge.** Open a PR from the Ridgeline branch, review, and merge to `main`
   once CI is green on its head.
2. **Choose the candidate.** Take the merged `main` commit whose
   *Arcade build and package* run succeeded in all three jobs. Record its
   commit, tree, run ID and the `release-v0.5.0-x86_64` artifact ID here.
3. **Verify the bundle** as for v0.4.0:
   - `sha256sum --check SHA256SUMS` passes for all four files;
   - `BUILD.txt` commit and tree match the candidate;
   - `.PKGINFO` reads `omarchy-retro-arcade`, `0.5.0-1`, `x86_64`;
   - the corresponding source archive matches `git archive` of the candidate;
   - the pinned Stockfish source and evaluation network are present.

   Record the four checksums here.
4. **Desktop acceptance on the Omarchy XPS.** Close Arcade and back up
   `~/.local/state`. Then:

   ```sh
   sha256sum --check SHA256SUMS
   sudo pacman -U ./omarchy-retro-arcade-0.5.0-1-x86_64.pkg.tar.zst
   omarchy-version
   omarchy-retro-arcade --version
   omarchy-retro-arcade --game ridgeline
   ```

   - Play at least First Terrace on Normal by mouse. Try the keyboard path too:
     1–4, arrows, Enter, U, Space, F and Esc.
   - Pause mid-wave, build while paused, press Ctrl+H, reopen, and quit and
     relaunch. The wave should resume paused and exactly as it was.
   - Switch to light and dark Omarchy themes during play.
   - Check that sound stops when leaving the game, Pinball still closes
     normally, and an existing FreeSki/Solitaire save still opens.
   - Note pacing and difficulty impressions. They feed
     [balance](../../games/ridgeline/docs/BALANCE.md), not this release gate.
   - Record hardware, exact Omarchy version, package checksum and results.
5. **Publish only when authorized.**
   - Tag **the exact candidate commit**, not a moving `main`, as `v0.5.0`.
   - Upload the five original bundle files unchanged.
   - Use [v0.5.0.md](v0.5.0.md) as the page description, removing its
     candidate sentence.
   - If a newer commit is chosen instead, use that commit's own CI bundle and
     repeat step 3. Never pair a newer tag with older assets.
6. **After publication**, update README's download/install section from 0.4.0
   to 0.5.0 and its released game count from fourteen to fifteen.

To rebuild locally, use a clean checkout and `packaging/build-arch.sh`, then
`packaging/prepare-release.sh dist/arch dist/release/v0.5.0`. Debug symbols
are a separate CI artifact. Official Omarchy package promotion and signing
remain separate operations.

## Known limitations to state at release

- Ridgeline's balance comes from a reference strategy, not human playtesting.
  The Hard tables on the early maps may be forgiving.
- Hands-on Ridgeline acceptance on Omarchy/Wayland is the remaining gate.
- The full old-version upgrade/rollback matrix and aarch64 are unverified.
