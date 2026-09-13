#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
prefix="${1:?Usage: scripts/install.sh DESTDIR [PREFIX]}${2:-/usr}"
# Explicit player payload; development evidence remains in the source repository.
while IFS=$'\t' read -r source destination; do
  [[ -z "$source" || "$source" == \#* ]] && continue
  if [[ "$source" == target/release/* ]]; then
    source="${CARGO_TARGET_DIR:-target}/${source#target/}"
  fi
  mode=644
  [[ "$destination" == bin/* || "$destination" == libexec/* ]] && mode=755
  install -Dm"$mode" "$source" "$prefix/$destination"
done < packaging/player-files.tsv
