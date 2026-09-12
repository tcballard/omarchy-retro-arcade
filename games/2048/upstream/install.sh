#!/usr/bin/env bash
# Install the native 2048 build on Omarchy / Arch (no sudo needed).
# Installs: ~/.local/bin/2048, ~/.local/share/applications/2048.desktop, ~/.local/share/icons/2048.png
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_SRC="$ROOT/src-tauri/target/release/2048"
BIN_DST="$HOME/.local/bin/2048"
DESKTOP_SRC="$ROOT/assets/2048.desktop"
DESKTOP_DST="$HOME/.local/share/applications/2048.desktop"
ICON_SRC="$ROOT/assets/icon.png"
ICON_DST="$HOME/.local/share/icons/2048.png"

if [[ ! -x "$BIN_SRC" ]]; then
  echo "Release binary not found at $BIN_SRC"
  echo "Build it first with: npm run build  (or ./build-native.sh)"
  exit 1
fi

mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" "$HOME/.local/share/icons"
install -Dm755 "$BIN_SRC" "$BIN_DST"
install -Dm644 "$ICON_SRC" "$ICON_DST"
install -Dm644 "$DESKTOP_SRC" "$DESKTOP_DST"

# Point the desktop entry at the installed binary
sed -i "s|^Exec=.*|Exec=$BIN_DST|" "$DESKTOP_DST"
sed -i "s|^Icon=.*|Icon=$ICON_DST|" "$DESKTOP_DST"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$HOME/.local/share/applications" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons" 2>/dev/null || true
fi

echo "Installed 2048 -> $BIN_DST"
echo "Launch with: 2048  (or from your app launcher / wofi / rofi)"
