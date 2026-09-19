# v0.4.0 release preparation

FreeSki is the fourteenth game. This candidate includes the host/shared-platform
and focus/input refactors, audio cleanup (#50), and Pinball shutdown (#51).
Workspace crates, Cargo.lock and the Arch player package advance to 0.4.0 together;
external dependency versions and save schemas do not change in this preparation.

## Readiness

- Gameplay: historical FreeSki sign-off at 5458784, scoped in
  [PLAYTESTS.md](../../games/freeski/docs/PLAYTESTS.md). Do not relabel it as a
  playtest of this candidate.
- Cleanup: #50 merged; #51 must pass its updated CI and merge before this PR.
- Candidate checks reproduced locally: 18 workspace versions align with
  PKGBUILD and Cargo.lock; external lock entries are unchanged. Locked Cargo
  metadata, workspace formatting, release/build shell syntax and diff checks
  pass. Full native/Arch CI is still required for this versioned source.
- Desktop acceptance: focused post-cleanup checks of FreeSki resume/focus/audio,
  game switching and Pinball close remain outstanding. Record exact
  `omarchy-version`, hardware and tested commit. Earlier general sign-off does
  not imply every Slalom medal/course or pursuit seed was individually tested.
- Upgrade: CI's same-package reinstall preserves saves; it is not a complete
  v0.3.0-to-v0.4.0 upgrade/rollback matrix. Back up existing state first.
- Publication: not authorized by this preparation. No tag or release is created.

## Build and assets

Use a clean checkout and `packaging/build-arch.sh`. The Arch CI job builds, tests,
installs and checks the package before running:

```sh
packaging/prepare-release.sh dist/arch dist/release/v0.4.0
```

Download `release-v0.4.0-x86_64` from the successful final-commit run. It contains:

- `omarchy-retro-arcade-0.4.0-1-x86_64.pkg.tar.zst`
- `omarchy-retro-arcade-0.4.0-corresponding-source.tar.gz`
- `RELEASE-NOTES.md`, `BUILD.txt` and `SHA256SUMS`

Debug symbols remain a separate artifact. The preparation script checks the
application archive against the exact clean git archive, validates package
identity/version/architecture, includes pinned Stockfish source/network and
refuses to overwrite an output directory. CI checks the installed version.
Digests will be generated from actual CI assets, not invented during preparation.

## Publication handoff

1. Merge #51, then this preparation PR. Rebuild the resulting main commit; a
   successful PR build does not identify a different merge commit's assets.
2. Confirm native, Arch and engine jobs succeed. Download its release artifact;
   run `sha256sum --check SHA256SUMS` and match BUILD.txt's source commit/tree to
   the intended tag target. Retain the corresponding source and licence files.
3. Record the focused desktop/upgrade results and preserve any untested cases in
   the release notes. aarch64 remains unverified.
4. On publication authorization, create v0.4.0 at that exact tested commit and
   upload the verified assets. Use [prepared notes](v0.4.0.md), remove candidate
   wording only after satisfying the gates, and link the published release from
   README. The existing v0.3.0 download stays valid until then.

Package downgrade does not undo state migration. Keep save backups and current
files. Official Omarchy package promotion/signing remains a separate operation.
