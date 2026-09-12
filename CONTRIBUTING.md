# Contributing to Omarchy Arcade

PRs, issues and playtesting are encouraged. Help make Arcade something you enjoy coming back to: fix a rough edge, improve a control, suggest a game or tell us what felt off. You do not need to write code to contribute.

## Report a problem or suggest something

[Choose an issue form](https://github.com/tcballard/omarchy-retro-arcade/issues/new/choose):

- **Bug report:** gameplay, installation, saves, sound or display failures.
- **Improvement or feature:** better behaviour in an existing game or the shared app.
- **New game proposal:** a game idea and its first playable scope.
- **Playtest feedback:** how the games feel on your actual desktop.

Search existing issues first and add useful evidence to an existing report when it matches. One problem per bug report makes fixes easier to track. Unknown technical details are fine; include what you can. Blank issues remain available for questions and anything the forms do not cover.

## Before a substantial change

Read [AGENTS.md](AGENTS.md), any instructions inside the game you are changing, and [DECISIONS.md](DECISIONS.md). Link an existing proposal or open one before investing heavily in a new game, dependency or shared behaviour change. Small fixes and documentation improvements can go straight to a PR.

Keep the established integration contract:

- One repository, native app window, desktop entry and package/release. Games are ordinary directories, not separately downloaded plugins.
- Use Rust for new application code. Preserve Pinball's upstream C++ engine and the approved game artwork.
- Preserve existing saves, settings and character/deck choices through upgrades. Installation must never delete user data.
- Preserve imported history, component licences and asset attribution. Include source and licence information for reused material.
- Keep gameplay available offline. Changes to optional online sharing must respect the boundaries recorded in DECISIONS.md.
- Record material integration choices in DECISIONS.md, referring to previous decisions when proposing a change.

## Build and check

Follow the [README build instructions](README.md#build) for dependencies and Stockfish setup. Use a feature branch from current `main` and keep commits focused.

The workspace checks used by CI are:

```sh
cargo fmt --all --check
cargo clippy --workspace --locked --all-targets -- -D warnings
REQUIRE_STOCKFISH=1 cargo test --workspace --locked --all-targets
scripts/build.sh
ctest --test-dir build/pinball --output-on-failure -R 'theme-palette|theme-path|authored-upstream-table'
desktop-file-validate packaging/omarchy-retro-arcade.desktop
```

`REQUIRE_STOCKFISH=1` requires a working Stockfish engine; a skipped engine test is not evidence that computer Chess works. See the [CI workflow](.github/workflows/arcade.yml) for native dependencies, game switching, save checks and render commands. On Arch, `packaging/build-arch.sh` builds from a clean committed checkout and does not install the result.

Check the affected behaviour in the running app, including returning to the collection. For changes to gameplay, storage or lifecycle, verify relevant pause/resume, close/reopen and save compatibility paths. For presentation changes, include actual screenshots and check light/dark themes, compact windows and 200% scale where relevant. Packaging changes need installation and upgrade/save evidence.

Report each check as passed, failed, skipped or blocked, with a reason for anything not run. Headless rendering and automated input checks are separate from hands-on Omarchy/Wayland acceptance; see [verification](docs/VERIFICATION.md) for current gaps. For documentation and template changes, report syntax, content and link checks and explicitly identify any runtime checks not run. CI still runs its configured gates.

## Open a pull request

Target `main` and complete the PR template. Explain the problem, resulting behaviour, verification and any compatibility effects. Link related issues or decisions; use `Closes #...` only for a complete fix. A draft PR is welcome for work in progress.

Keep unrelated changes in separate PRs. Make it clear what is ready to review and what still needs playtesting. Suggestions and constructive review are welcome throughout.
