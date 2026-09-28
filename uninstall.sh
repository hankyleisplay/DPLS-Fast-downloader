#!/usr/bin/env bash
set -e

# ==============================================================================
# DPLS-Fast Linux 反安裝程式 (Uninstaller)
# ==============================================================================

BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICONS_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
ICONS_128_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
PIXMAPS_DIR="$HOME/.local/share/pixmaps"
AUTOSTART_DIR="$HOME/.config/autostart"

DESKTOP_DIR="$HOME/Desktop"
if command -v xdg-user-dir >/dev/null 2>&1; then
  XDG_DESK="$(xdg-user-dir DESKTOP 2>/dev/null || true)"
  if [ -n "$XDG_DESK" ] && [ -d "$XDG_DESK" ]; then
    DESKTOP_DIR="$XDG_DESK"
  fi
fi

echo ""
echo -e "\033[1;31m======================================================================\033[0m"
echo -e "\033[1;33m  🗑️ DPLS-Fast 極速多線程傳輸加速器 - 移除程式 (Linux)\033[0m"
echo -e "\033[1;31m======================================================================\033[0m"
echo ""

# 1. Terminate running instances
echo "🛑 正在停止背景常駐服務..."
pkill -9 -f dpls-gui 2>/dev/null || true
pkill -9 -f dpls 2>/dev/null || true

# 2. Remove executables
echo "🗑️ 正在移除執行檔..."
rm -f "$BIN_DIR/dpls" "$BIN_DIR/dpls-gui"

# 3. Remove icons
echo "🗑️ 正在移除系統圖示..."
rm -f "$ICONS_DIR/dpls-fast.png"
rm -f "$ICONS_128_DIR/dpls-fast.png"
rm -f "$PIXMAPS_DIR/dpls-fast.png"

# 4. Remove launchers, desktop shortcuts and local extension
echo "🗑️ 正在移除應用程式選單、桌面捷徑、自啟檔案與擴充套件目錄..."
rm -f "$APPS_DIR/dpls-fast.desktop"
rm -f "$DESKTOP_DIR/dpls-fast.desktop"
rm -f "$AUTOSTART_DIR/dpls-fast.desktop"
rm -rf "$HOME/.local/share/dpls-fast"

# 5. Refresh desktop and icon database
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo -e "\033[1;32m✅ DPLS-Fast 已完全從系統中移除乾淨！\033[0m"
echo ""
