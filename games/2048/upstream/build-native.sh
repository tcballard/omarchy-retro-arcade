#!/usr/bin/env bash
# Build the native 2048 app (Tauri release) on Omarchy / Arch.
set -euo pipefail
cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install rust: sudo pacman -S rust"
  exit 1
fi

if [[ ! -d node_modules ]]; then
  npm install
fi

# Generate Tauri icons from assets/icon.png (creates 32x32, 128x128, ico, icns)
npx tauri icon assets/icon.png 2>/dev/null || npx tauri icon --help 2>&1 | head -5 || true

npm run build
echo "Build done. Install with: ./install.sh"
