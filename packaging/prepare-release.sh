#!/usr/bin/env bash
set -euo pipefail
# Prepare existing, tested build outputs. Never tags, uploads or installs.
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
input="$(realpath "${1:-dist/arch}")"
version="$(sed -n 's/^pkgver=//p' packaging/PKGBUILD)"
output="$(realpath -m "${2:-dist/release/v$version}")"
[[ -z "$(git status --porcelain --untracked-files=normal)" ]] || { echo 'Commit source changes first.' >&2; exit 1; }
[[ ! -e "$output" ]] || { echo 'Release output already exists; choose a fresh directory.' >&2; exit 1; }
python3 - <<'PY'
import tomllib
from pathlib import Path
workspace = tomllib.loads(Path('Cargo.toml').read_text())
version = workspace['workspace']['package']['version']
assert f'pkgver={version}\n' in Path('packaging/PKGBUILD').read_text()
lock = tomllib.loads(Path('Cargo.lock').read_text())
assert all(p['version'] == version for p in lock['package'] if 'source' not in p)
assert Path(f'docs/releases/v{version}.md').is_file()
PY
work="$(mktemp -d)"
trap 'rm -rf -- "$work"' EXIT
# The corresponding app archive must be exactly the committed build input.
git archive --format=tar.gz --prefix=omarchy-retro-arcade/ HEAD > "$work/expected.tar.gz"
cmp "$work/expected.tar.gz" "$input/sources/omarchy-retro-arcade-source.tar.gz"
shopt -s nullglob
packages=("$input"/omarchy-retro-arcade-[0-9]*.pkg.tar.zst)
[[ ${#packages[@]} -eq 1 ]] || { echo 'Expected exactly one player package.' >&2; exit 1; }
bsdtar -xOf "${packages[0]}" .PKGINFO > "$work/pkginfo"
grep -Fxq 'pkgname = omarchy-retro-arcade' "$work/pkginfo"
grep -Fxq "pkgver = $version-1" "$work/pkginfo"
grep -Fxq 'arch = x86_64' "$work/pkginfo"
[[ "$(basename "${packages[0]}")" == "omarchy-retro-arcade-$version-1-x86_64.pkg.tar.zst" ]]
# Require the pinned engine source and network, not only application source.
[[ -s "$input/sources/stockfish-59aae690f91d6f69aac194f447d84b4a2c3be778.tar.gz" ]]
[[ -s "$input/sources/nn-1a298aa575a0.nnue" ]]
mkdir "$work/bundle"
cp "${packages[0]}" "$work/bundle/"
tar -czf "$work/bundle/omarchy-retro-arcade-$version-corresponding-source.tar.gz" -C "$input" sources
cp "docs/releases/v$version.md" "$work/bundle/RELEASE-NOTES.md"
{
  echo "version=$version"
  echo "commit=$(git rev-parse HEAD)"
  echo "tree=$(git rev-parse HEAD^{tree})"
  echo 'architecture=x86_64'
  echo "workflow_run=${GITHUB_RUN_ID:-local}"
  echo "workflow_attempt=${GITHUB_RUN_ATTEMPT:-local}"
  echo "app_source_sha256=$(sha256sum "$work/expected.tar.gz" | cut -d ' ' -f1)"
} > "$work/bundle/BUILD.txt"
(cd "$work/bundle" && sha256sum ./* > SHA256SUMS && sha256sum --check SHA256SUMS)
mkdir -p "$(dirname "$output")"
mv "$work/bundle" "$output"
printf 'Prepared v%s assets in %s; nothing published.\n' "$version" "$output"
