# v0.4.0 publication record

FreeSki adds the fourteenth game. The tested candidate is
`7d6e5f42b2e451ea08649eca577675990e58e9d8`, tree
`22ec5860c18f05743069bfb36f581e490ab362eb`.
The host/platform, focus/input, audio-worker and Pinball shutdown changes are
merged. All 18 local Cargo packages and the Arch package agree on 0.4.0.

## Published 21 September 2026

[v0.4.0 is published](https://github.com/tcballard/omarchy-retro-arcade/releases/tag/v0.4.0)
at the verified candidate below, with all five original CI bundle files.
Published asset digests match the locally checked files. PR #54 supplied updated
release-page copy; its documentation commit is not the binary's tag target.

Tom reported testing on his Omarchy XPS: “looks awesome (works really well)”,
and explicitly approved release. This is human-reported acceptance. Exact OS
version, individual cases and screenshots were not supplied in this thread;
no additional coverage is inferred. Full old-version upgrade/rollback and
ARM support remain unverified. The checklist below is retained as preparation
history, not outstanding publication authorization.

## Verified candidate

[Run 35525789201](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/35525789201)
completed successfully on 20 September 2026 at that exact commit:

- Native: formatting, strict Clippy, workspace tests with Stockfish, FreeSki
  headless tests, fourteen-game switching, FreeSki controls/resume/layouts,
  mouse navigation, Pinball controls/resize and the other game checks.
- Arch: complete package build, install verification, installed FreeSki evidence,
  same-package save preservation and release asset/checksum preparation.
- Tanks engine: engine tests, Clippy and shared-platform/headless boundaries.

These are existing CI results inspected on 21 September, not fresh local runtime
tests. The current preparation changes documentation only; they do not transfer
that evidence to a new source commit.

On 21 September the release ZIP was downloaded and independently checked:

- Artifact `10609568524`, `release-v0.4.0-x86_64`, SHA-256
  `6c3a159cd38e9bea7fb3e5127e33028829c64807fbe9c43ded85eb6962352269`.
- `sha256sum --check SHA256SUMS`: all four files passed.
- BUILD.txt commit/tree match the candidate above; workflow attempt is 1.
- `.PKGINFO`: `omarchy-retro-arcade`, `0.4.0-1`, `x86_64`.
- The corresponding application archive's members, file bytes, modes and
  symlinks match `git archive` of that commit. Local gzip bytes differ from CI;
  the embedded archive itself matches BUILD.txt's recorded SHA-256.
- Pinned Stockfish source and evaluation network are included.

Verification ran with Python 3, git, GNU tar/zstd and sha256sum on Linux x86_64.
No local Rust build or Omarchy desktop playtest was run in this preparation.

### Bundle checksums

```text
b1cdf89277f9172e550370463c79b10d59f0815f7cda707ee5867920bb6c2cf6  BUILD.txt
3cca0c09c3a812ffc95658e577b1a9c317792906012dfceca956f2105ffaa626  RELEASE-NOTES.md
33fba0ba9da1edf9a81f9989cba6811ddbfc5f0f99ce73d3a2f8e340bf12323a  omarchy-retro-arcade-0.4.0-1-x86_64.pkg.tar.zst
aebee600ef6674a7f007c8f9f3adb358efd9723d6d31636af8edc8fe88f4cc35  omarchy-retro-arcade-0.4.0-corresponding-source.tar.gz
```

The bundle's RELEASE-NOTES.md is the original build-time copy. Keep its bytes
and checksum intact. Use [the revised release-page copy](v0.4.0.md) for the GitHub
release description; it includes the gameplay image and contributor credit.

## Historical desktop checklist

Historical FreeSki gameplay approval is preserved in
[PLAYTESTS.md](../../games/freeski/docs/PLAYTESTS.md), scoped to the earlier build.
Final-package desktop acceptance remains unresolved. On the Omarchy XPS:

1. Close Arcade and back up existing game state using [save locations](../MIGRATION.md).
2. Download the verified [release artifact](https://github.com/tcballard/omarchy-retro-arcade/actions/runs/35525789201/artifacts/10609568524)
   and extract it. From that directory run:

   ```sh
   sha256sum --check SHA256SUMS
   sudo pacman -U ./omarchy-retro-arcade-0.4.0-1-x86_64.pkg.tar.zst
   omarchy-version
   omarchy-retro-arcade --version
   omarchy-retro-arcade --game freeski
   ```

3. Play Free Ski with keyboard and mouse; toggle F, brake, pause, close and reopen.
   Confirm the run resumes paused and held input does not survive focus loss.
4. Return to Arcade and switch games. Check sound stops when leaving a game,
   and Pinball closes normally. Reopen existing saves and check records/settings.
5. Record hardware, exact Omarchy version, package checksum and observed results.

The full old-version upgrade/rollback matrix and aarch64 remain unverified.
A package downgrade does not undo save migration; retain backups and current
files. Uninstalling the package should preserve state directories.

## Historical publication handoff

No v0.4.0 tag or public release existed when checked on 21 September.

1. Finish and record the focused desktop check above.
2. Publish the reviewed candidate only when authorized. Tag **the exact commit
   above**, not a moving main branch, and upload the five original bundle files.
   Documentation updates in this PR can supply the release-page description
   without changing the tested binary or its corresponding source.
3. If choosing a newer source commit instead, use its own successful CI bundle
   and repeat identity/checksum checks. Never pair a newer tag with these assets.
4. Remove candidate wording from the release-page copy only after the gates are
   satisfied. Keep the original bundled notes identified as build-time notes.
5. After publication, change README's download/install version from 0.3.0 to
   0.4.0 and its released game count from thirteen to fourteen.

To rebuild, use a clean checkout and `packaging/build-arch.sh`, then
`packaging/prepare-release.sh dist/arch dist/release/v0.4.0`.
Debug symbols are a separate CI artifact. Official Omarchy package promotion
and signing remain separate operations.
