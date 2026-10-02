#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "📦 開始打包 DPLS-Fast 發行套件..."

DIST_DIR="$ROOT_DIR/dist"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 1. Package Browser Extension
echo "🧩 打包瀏覽器擴充功能 (Chrome/Edge/Brave)..."
cd "$ROOT_DIR/extension"
zip -r "$DIST_DIR/dpls-fast-extension-v1.3.zip" ./* -x "*.DS_Store"
cp "$DIST_DIR/dpls-fast-extension-v1.3.zip" "$DIST_DIR/dpls-fast-extension-v1.2.zip"

# 2. Package Linux Binaries & Self-contained Installer
echo "🐧 打包 Linux 原生執行檔與安裝程式..."
cd "$ROOT_DIR"
RELEASE_DIR="$DIST_DIR/dpls-fast-linux-x86_64"
mkdir -p "$RELEASE_DIR"

cp "$ROOT_DIR/dpls" "$RELEASE_DIR/dpls"
cp "$ROOT_DIR/dpls-gui" "$RELEASE_DIR/dpls-gui"
cp "$ROOT_DIR/install.sh" "$RELEASE_DIR/install.sh"
cp "$ROOT_DIR/update.sh" "$RELEASE_DIR/update.sh"
cp "$ROOT_DIR/uninstall.sh" "$RELEASE_DIR/uninstall.sh"
mkdir -p "$RELEASE_DIR/src-tauri/icons"
cp "$ROOT_DIR/src-tauri/icons/icon.png" "$RELEASE_DIR/src-tauri/icons/icon.png"
cp -rf "$ROOT_DIR/extension" "$RELEASE_DIR/"
chmod +x "$RELEASE_DIR/dpls" "$RELEASE_DIR/dpls-gui" "$RELEASE_DIR/install.sh" "$RELEASE_DIR/update.sh" "$RELEASE_DIR/uninstall.sh"

tar -czvf "$DIST_DIR/dpls-fast-linux-x86_64.tar.gz" -C "$DIST_DIR" dpls-fast-linux-x86_64
rm -rf "$RELEASE_DIR"

# 3. Copy Windows Installer & Updater Scripts to dist
cp "$ROOT_DIR/scripts/windows-installer.ps1" "$DIST_DIR/windows-installer.ps1"
cp "$ROOT_DIR/scripts/windows-updater.ps1" "$DIST_DIR/windows-updater.ps1"
cp "$ROOT_DIR/scripts/windows-uninstaller.ps1" "$DIST_DIR/windows-uninstaller.ps1"

echo "✅ 打包完成！輸出檔案位於 $DIST_DIR："
ls -lh "$DIST_DIR"
