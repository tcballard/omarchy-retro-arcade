#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
prefix="${1:?Usage: scripts/install.sh DESTDIR [PREFIX]}${2:-/usr}"
install -Dm755 "${CARGO_TARGET_DIR:-target}/release/omarchy-retro-arcade" "$prefix/bin/omarchy-retro-arcade"
install -Dm755 build/pinball/bin/omarchy-spacecadet-game "$prefix/libexec/omarchy-retro-arcade/circuit"
install -Dm644 games/pinball/assets/circuit/table.png "$prefix/share/omarchy-retro-arcade/circuit/table.png"
install -Dm644 packaging/omarchy-retro-arcade.desktop "$prefix/share/applications/omarchy-retro-arcade.desktop"
install -Dm644 packaging/omarchy-retro-arcade.svg "$prefix/share/icons/hicolor/scalable/apps/omarchy-retro-arcade.svg"
install -Dm644 LICENSE "$prefix/share/licenses/omarchy-retro-arcade/LICENSE"
for game in chess solitaire scram invaders pinball stack snake bubble blast 2048 shatter; do
  install -Dm644 "games/$game/LICENSE" "$prefix/share/licenses/omarchy-retro-arcade/$game/LICENSE"
  for doc in THIRD_PARTY.md THIRD_PARTY_NOTICES.md THIRD_PARTY_LICENSES.txt NOTICE; do
    if [[ -f "games/$game/$doc" ]]; then install -Dm644 "games/$game/$doc" "$prefix/share/licenses/omarchy-retro-arcade/$game/$doc"; fi
  done
done
install -Dm644 shared/presentation/assets/README.md "$prefix/share/doc/omarchy-retro-arcade/CABINET-ARTWORK.md"
install -Dm644 docs/MIGRATION.md "$prefix/share/doc/omarchy-retro-arcade/MIGRATION.md"
# Full provenance includes artwork-specific notices and authorship.
cp -R docs "$prefix/share/doc/omarchy-retro-arcade/collection"
for game in chess solitaire scram invaders pinball stack snake bubble blast 2048 shatter; do
  mkdir -p "$prefix/share/doc/omarchy-retro-arcade/$game"
  if [[ -d "games/$game/docs" ]]; then cp -R "games/$game/docs" "$prefix/share/doc/omarchy-retro-arcade/$game/"; fi
  if [[ -d "games/$game/assets" ]]; then find "games/$game/assets" -type f \( -iname '*license*' -o -iname '*readme*' \) -exec cp --parents '{}' "$prefix/share/doc/omarchy-retro-arcade/$game/" \; ; fi
done
