# Maintainer: avibarit
pkgname=2048-game
pkgver=1.0.0
pkgrel=1
pkgdesc="Classic 2048 puzzle game — native desktop app (Tauri + WebKitGTK)"
arch=('x86_64')
url="https://github.com/avibarit/2048"
license=('MIT')
depends=('webkit2gtk-4.1' 'gtk3' 'libsoup3' 'hicolor-icon-theme')
makedepends=('rust' 'cargo' 'nodejs' 'npm')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/2048-$pkgver"
  npm ci --no-audit --no-fund
  npm run build
}

package() {
  cd "$srcdir/2048-$pkgver"
  install -Dm755 "src-tauri/target/release/2048" "$pkgdir/usr/bin/2048"
  install -Dm644 "assets/2048.desktop" "$pkgdir/usr/share/applications/2048.desktop"
  install -Dm644 "assets/icon.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/2048.png"
  for s in 32 128; do
    if [[ -f "src-tauri/icons/${s}x${s}.png" ]]; then
      install -Dm644 "src-tauri/icons/${s}x${s}.png" "$pkgdir/usr/share/icons/hicolor/${s}x${s}/apps/2048.png"
    fi
  done
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
