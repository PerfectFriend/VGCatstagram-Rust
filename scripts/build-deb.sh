#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════
# Debian Package Builder for VanGoghCat Rust Core v2.0 («Renaissance»)
# ═══════════════════════════════════════════════════════════════════════

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="2.0.0"
ARCH="$(dpkg --print-architecture 2>/dev/null || echo "amd64")"
PKG_NAME="vangoghcat-core_${VERSION}_${ARCH}"

echo "🦀 Building VanGoghCat Release Binary (CLI & Daemon)..."
cd "${REPO_ROOT}"
cargo build --release -p vangoghcat-cli

TARGET_DIR="$(python3 -c "import json, subprocess; print(json.loads(subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--no-deps']))['target_directory'])")"
BIN_SRC="${TARGET_DIR}/release/vangoghcat"

if [ ! -f "${BIN_SRC}" ]; then
    echo "❌ Error: Binary not found at ${BIN_SRC}"
    exit 1
fi

BUILD_DIR="${TARGET_DIR}/deb_dist/${PKG_NAME}"
echo "📦 Preparing Debian package layout in ${BUILD_DIR}..."
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/usr/local/bin"
mkdir -p "${BUILD_DIR}/lib/systemd/system"
mkdir -p "${BUILD_DIR}/DEBIAN"

cp "${BIN_SRC}" "${BUILD_DIR}/usr/local/bin/vangoghcat"
chmod +x "${BUILD_DIR}/usr/local/bin/vangoghcat"

cp "${REPO_ROOT}/systemd/vangoghcat.service" "${BUILD_DIR}/lib/systemd/system/vangoghcat.service"

cat <<EOF > "${BUILD_DIR}/DEBIAN/control"
Package: vangoghcat-core
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: VanGoghCat Guild <guild@vangoghcat.eth>
Description: Sovereign Rust Core for Van Gogh CryptoCats Ecosystem
 High-performance zero-copy P2P daemon, Sled KV database, 8 GB quota guard,
 and Axum REST/WebSocket signaling relay.
EOF

cat <<EOF > "${BUILD_DIR}/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
fi
echo "✅ VanGoghCat Core v2.0 installed successfully!"
echo "   Run 'vangoghcat info' or 'vangoghcat start'"
exit 0
EOF
chmod 755 "${BUILD_DIR}/DEBIAN/postinst"

echo "🔨 Assembling .deb package..."
mkdir -p "${REPO_ROOT}/dist"
dpkg-deb --build --root-owner-group "${BUILD_DIR}" "${REPO_ROOT}/dist/${PKG_NAME}.deb"

echo "🎉 Debian package created: ${REPO_ROOT}/dist/${PKG_NAME}.deb"
ls -lh "${REPO_ROOT}/dist/${PKG_NAME}.deb"
