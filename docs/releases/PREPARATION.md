# Release preparation

The refreshed candidate includes main through #38 (`26ab74c014c490af5112fc2fecd57604627d06d8`),
including its high-score dialog and native paste regression checks.

For v0.2.0, workspace and Arch versions advance together. No game rules, save
schemas, imported version history or external dependency versions change.

Reproduced locally for the preparation diff: TOML version/lock consistency checks
(15 workspace entries; all external entries unchanged), YAML parsing, shell
syntax and whitespace checks. A synthetic package fixture exercises source
mismatch and old-version rejection, output overwrite refusal, successful bundling
and checksum verification. That fixture contains no executable and is not an
Arch installation test. Runtime/package evidence for the merged baseline is
linked in [v0.2.0 notes](v0.2.0.md); versioned CI is separate.

## Candidate assets

After the Arch job builds, installs and checks its package, it runs:

```sh
packaging/prepare-release.sh dist/arch dist/release/v0.2.0
```

The output contains the x86_64 player package, a corresponding-source archive
(application source, pinned Stockfish source and network), release notes,
BUILD.txt and SHA256SUMS. Debug symbols remain in their separate CI artifact.
The script checks the source archive against the clean checkout's exact git
archive, checks package name/version/architecture and refuses existing output.
CI additionally checks the installed executable's version. This prepares files;
it does not create tags, releases or uploads outside Actions artifacts.

## Publication handoff

1. Review and merge the preparation PR. Confirm all jobs succeed on the intended
   final tag commit. If rebasing or squashing changes its SHA, rebuild that commit
   and use its assets; do not relabel a previous build's BUILD.txt.
2. Download `release-v0.2.0-x86_64` from that successful run, extract it and run
   `sha256sum --check SHA256SUMS`. Match BUILD.txt's commit to the tag target.
3. Record any live Omarchy testing, including exact `omarchy-version`, hardware,
   audio, window/focus behaviour and old-package upgrade. Keep untested cases in
   the release notes; CI's same-package reinstall is not a migration matrix.
4. On publication authorization, tag that exact commit `v0.2.0` and create the
   v0.2.0 release without a beta suffix. Upload the checked bundle files together,
   retaining the corresponding source. Remove preparation-status wording from
   the public description and link the final assets from README.

Publishing an app release does not promote it into the official Omarchy package
repository. No signature or live desktop acceptance is implied by SHA-256 digests.
