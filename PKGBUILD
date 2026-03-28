# Maintainer: r3dlight
pkgname=qobuz-tui
pkgver=0.5.0
pkgrel=1
pkgdesc="A terminal-based Qobuz music player"
arch=('x86_64')
url="https://github.com/r3dlight/qobuz-tui"
license=('GPL-3.0-only')
depends=('alsa-lib')
makedepends=('rust' 'cargo')

build() {
    cd "$startdir"
    cargo build --release
}

package() {
    cd "$startdir"
    install -Dm755 "target/release/qobuz-tui" "$pkgdir/usr/bin/qobuz-tui"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/qobuz-tui/README.md"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/qobuz-tui/LICENSE"
}
