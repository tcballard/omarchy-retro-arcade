# FreeSki project instructions

FreeSki is an existing Rust library inside Arcade. Preserve the one-window/package
contract, existing game order, saves and approved artwork in root AGENTS.md.

## Read according to the task

- Start with [README.md](README.md) and [docs/PLAN.md](docs/PLAN.md) for current
  capabilities and next work. Do not repeat completed project setup.
- For cross-module design or new modes, read [docs/SYSTEM.md](docs/SYSTEM.md).
  The next playtest is scoped in [docs/NEXT.md](docs/NEXT.md).
- For behavior changes, read [docs/RULES.md](docs/RULES.md) and the active revision
  in [docs/TUNING.md](docs/TUNING.md). Tyler liked the rules-2 speed and turning.
- For checks and closeout, use [docs/VERIFICATION.md](docs/VERIFICATION.md) and
  root CONTRIBUTING.md. Historical runs are evidence for their named revision.
- For scope questions, consult [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md), the
  issue snapshot. MILESTONE-1.md is historical; current plans supersede its agenda.

## Implementation contracts

- Keep fixed-tick simulation independent of UI, audio, filesystem and wall time.
  Full-run gameplay, replay and reference evidence should converge on the shared
  session path described in SYSTEM.md. Direct engine calls remain useful for focused
  physics tests; never add test-only movement or collision exemptions.
- State owns outcomes. Rendering/resize cannot change physics, terrain or records.
  Terrain is versioned and reproducible; cache and cosmetic effects stay bounded.
- Preserve supported saves through explicit validated migration and retain the
  original. Unsupported/corrupt files keep the existing explicit recovery flow.
  Do not reset actual user state to make a test or screenshot convenient.
- Every behavior revision needs a discriminating check and current rules/tuning.
  Record material integration choices in root DECISIONS.md and actual results in
  VERIFICATION.md. Keep human observations separate from reference feasibility.
- Use a feature branch and focused commits. Run applicable combined gates after
  integration; repeat passing checks only for changed source, a failure or a new
  concern. Documentation-only work needs content/link/diff checks, not a game build.
