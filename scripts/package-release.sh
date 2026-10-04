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
if command -v zip >/dev/null 2>&1; then
    zip -r "$DIST_DIR/dpls-fast-extension-v1.3.zip" ./* -x "*.DS_Store"
    cp "$DIST_DIR/dpls-fast-extension-v1.3.zip" "$DIST_DIR/dpls-fast-extension-v1.2.zip"
fi

# 2. Package Linux Binaries & Self-contained Installer
echo "🐧 打包 Linux 原生執行檔與安裝程式..."
cd "$ROOT_DIR"
RELEASE_DIR="$DIST_DIR/dpls-fast-linux-x86_64"
mkdir -p "$RELEASE_DIR"

if [ -f "$ROOT_DIR/dpls" ]; then cp "$ROOT_DIR/dpls" "$RELEASE_DIR/dpls"; fi
if [ -f "$ROOT_DIR/dpls-gui" ]; then cp "$ROOT_DIR/dpls-gui" "$RELEASE_DIR/dpls-gui"; fi
if [ -f "$ROOT_DIR/install.sh" ]; then cp "$ROOT_DIR/install.sh" "$RELEASE_DIR/install.sh"; fi
if [ -f "$ROOT_DIR/update.sh" ]; then cp "$ROOT_DIR/update.sh" "$RELEASE_DIR/update.sh"; fi
if [ -f "$ROOT_DIR/uninstall.sh" ]; then cp "$ROOT_DIR/uninstall.sh" "$RELEASE_DIR/uninstall.sh"; fi
mkdir -p "$RELEASE_DIR/src-tauri/icons"
if [ -f "$ROOT_DIR/src-tauri/icons/icon.png" ]; then cp "$ROOT_DIR/src-tauri/icons/icon.png" "$RELEASE_DIR/src-tauri/icons/icon.png"; fi
if [ -d "$ROOT_DIR/extension" ]; then cp -rf "$ROOT_DIR/extension" "$RELEASE_DIR/"; fi
chmod +x "$RELEASE_DIR"/dpls* "$RELEASE_DIR"/*.sh 2>/dev/null || true

if command -v tar >/dev/null 2>&1; then
    tar -czvf "$DIST_DIR/dpls-fast-linux-x86_64.tar.gz" -C "$DIST_DIR" dpls-fast-linux-x86_64
fi
rm -rf "$RELEASE_DIR"

# 3. Package Windows Distribution Archive & Scripts
echo "🪟 打包 Windows 原生執行檔與安裝套件..."
WIN_RELEASE_DIR="$DIST_DIR/dpls-fast-windows-x86_64"
mkdir -p "$WIN_RELEASE_DIR/scripts"

for f in dpls.exe dpls-gui.exe dpls-desktop.exe DPLS-Fast.exe; do
    if [ -f "$ROOT_DIR/$f" ]; then cp "$ROOT_DIR/$f" "$WIN_RELEASE_DIR/$f"; fi
    if [ -f "$ROOT_DIR/target/release/$f" ]; then cp "$ROOT_DIR/target/release/$f" "$WIN_RELEASE_DIR/$f"; fi
    if [ -f "$ROOT_DIR/src-tauri/target/release/$f" ]; then cp "$ROOT_DIR/src-tauri/target/release/$f" "$WIN_RELEASE_DIR/$f"; fi
done

for setup_exe in "$ROOT_DIR"/*setup*.exe "$ROOT_DIR/src-tauri/target/release/bundle/nsis"/*setup*.exe; do
    if [ -f "$setup_exe" ]; then cp "$setup_exe" "$WIN_RELEASE_DIR/"; fi
done

cp "$ROOT_DIR/install.bat" "$WIN_RELEASE_DIR/install.bat"
cp "$ROOT_DIR/install.ps1" "$WIN_RELEASE_DIR/install.ps1"
cp "$ROOT_DIR/uninstall.bat" "$WIN_RELEASE_DIR/uninstall.bat"
cp "$ROOT_DIR/uninstall.ps1" "$WIN_RELEASE_DIR/uninstall.ps1"
cp "$ROOT_DIR/update.bat" "$WIN_RELEASE_DIR/update.bat"
cp "$ROOT_DIR/update.ps1" "$WIN_RELEASE_DIR/update.ps1"
cp "$ROOT_DIR/scripts/windows-installer.ps1" "$WIN_RELEASE_DIR/scripts/windows-installer.ps1"
cp "$ROOT_DIR/scripts/windows-updater.ps1" "$WIN_RELEASE_DIR/scripts/windows-updater.ps1"
cp "$ROOT_DIR/scripts/windows-uninstaller.ps1" "$WIN_RELEASE_DIR/scripts/windows-uninstaller.ps1"
if [ -f "$ROOT_DIR/scripts/launcher.cs" ]; then cp "$ROOT_DIR/scripts/launcher.cs" "$WIN_RELEASE_DIR/scripts/launcher.cs"; fi

mkdir -p "$WIN_RELEASE_DIR/src-tauri/icons"
if [ -f "$ROOT_DIR/src-tauri/icons/icon.ico" ]; then cp "$ROOT_DIR/src-tauri/icons/icon.ico" "$WIN_RELEASE_DIR/src-tauri/icons/icon.ico"; fi
if [ -d "$ROOT_DIR/extension" ]; then cp -rf "$ROOT_DIR/extension" "$WIN_RELEASE_DIR/"; fi
if [ -d "$ROOT_DIR/web" ]; then cp -rf "$ROOT_DIR/web" "$WIN_RELEASE_DIR/"; fi

if command -v zip >/dev/null 2>&1; then
    cd "$DIST_DIR"
    zip -r "dpls-fast-windows-x86_64.zip" dpls-fast-windows-x86_64
fi
rm -rf "$WIN_RELEASE_DIR"

# Copy standalone scripts to dist
cp "$ROOT_DIR/scripts/windows-installer.ps1" "$DIST_DIR/windows-installer.ps1"
cp "$ROOT_DIR/scripts/windows-updater.ps1" "$DIST_DIR/windows-updater.ps1"
cp "$ROOT_DIR/scripts/windows-uninstaller.ps1" "$DIST_DIR/windows-uninstaller.ps1"
cp "$ROOT_DIR/install.bat" "$DIST_DIR/install.bat"
cp "$ROOT_DIR/install.ps1" "$DIST_DIR/install.ps1"

echo "✅ 打包完成！輸出檔案位於 $DIST_DIR："
ls -lh "$DIST_DIR"
