#!/usr/bin/env bash
set -e

# ==============================================================================
# DPLS-Fast Linux 一鍵安裝程式 (Installer with Desktop Shortcuts & Autostart)
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICONS_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
ICONS_128_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
PIXMAPS_DIR="$HOME/.local/share/pixmaps"
AUTOSTART_DIR="$HOME/.config/autostart"

# Detect localized Desktop directory
DESKTOP_DIR="$HOME/Desktop"
if command -v xdg-user-dir >/dev/null 2>&1; then
  XDG_DESK="$(xdg-user-dir DESKTOP 2>/dev/null || true)"
  if [ -n "$XDG_DESK" ] && [ -d "$XDG_DESK" ]; then
    DESKTOP_DIR="$XDG_DESK"
  fi
fi

echo ""
echo -e "\033[1;36m======================================================================\033[0m"
echo -e "\033[1;32m  🚀 DPLS-Fast 極速多線程傳輸加速器 - 安裝程式 (Linux)\033[0m"
echo -e "\033[1;36m======================================================================\033[0m"
echo ""

# 1. Check binaries
CLI_SRC=""
GUI_SRC=""

if [ -f "$SCRIPT_DIR/dpls" ]; then
  CLI_SRC="$SCRIPT_DIR/dpls"
elif [ -f "$SCRIPT_DIR/dist/dpls-linux-x86_64" ]; then
  CLI_SRC="$SCRIPT_DIR/dist/dpls-linux-x86_64"
fi

if [ -f "$SCRIPT_DIR/dpls-gui" ]; then
  GUI_SRC="$SCRIPT_DIR/dpls-gui"
elif [ -f "$SCRIPT_DIR/dist/dpls-gui-linux-x86_64" ]; then
  GUI_SRC="$SCRIPT_DIR/dist/dpls-gui-linux-x86_64"
fi

if [ -z "$CLI_SRC" ] || [ -z "$GUI_SRC" ]; then
  echo -e "\033[1;33m⚠️ 未找到已編譯的執行檔，正在使用 Cargo 建置發行版本...\033[0m"
  cd "$SCRIPT_DIR"
  cargo build --release
  cd "$SCRIPT_DIR/src-tauri"
  cargo build --release
  cd "$SCRIPT_DIR"
  CLI_SRC="$SCRIPT_DIR/target/release/dpls"
  GUI_SRC="$SCRIPT_DIR/target/release/dpls-desktop"
fi

# 2. Deploy Executables
echo "📦 正在安裝執行檔至 $BIN_DIR ..."
mkdir -p "$BIN_DIR"
cp -f "$CLI_SRC" "$BIN_DIR/dpls"
cp -f "$GUI_SRC" "$BIN_DIR/dpls-gui"
chmod +x "$BIN_DIR/dpls" "$BIN_DIR/dpls-gui"

# 3. Deploy High-Resolution Icons
echo "🎨 正在安裝高解析度系統圖示..."
mkdir -p "$ICONS_DIR" "$ICONS_128_DIR" "$PIXMAPS_DIR"

if [ -f "$SCRIPT_DIR/src-tauri/icons/icon.png" ]; then
  cp -f "$SCRIPT_DIR/src-tauri/icons/icon.png" "$ICONS_DIR/dpls-fast.png"
  cp -f "$SCRIPT_DIR/src-tauri/icons/icon.png" "$PIXMAPS_DIR/dpls-fast.png"
fi

if [ -f "$SCRIPT_DIR/extension/icons/icon128.png" ]; then
  cp -f "$SCRIPT_DIR/extension/icons/icon128.png" "$ICONS_128_DIR/dpls-fast.png"
elif [ -f "$SCRIPT_DIR/src-tauri/icons/icon.png" ]; then
  cp -f "$SCRIPT_DIR/src-tauri/icons/icon.png" "$ICONS_128_DIR/dpls-fast.png"
fi

# 4. Create Desktop Application Launcher
echo "🖥️ 正在建立應用程式選單啟動器..."
mkdir -p "$APPS_DIR"
DESKTOP_FILE_CONTENT="[Desktop Entry]
Version=1.0
Type=Application
Name=DPLS-Fast
GenericName=檔案傳輸加速系統
GenericName[en]=File Transfer Accelerator
GenericName[zh_CN]=文件传输加速系统
GenericName[ja]=高速ダウンロード加速器
Comment=動態多路分段極速傳輸系統 (64 併發連線加速)
Comment[en]=High-speed dynamic multi-segment downloader with 64+ multiplexed connections
Comment[zh_CN]=动态多路分段极速传输系统 (64 并发连接加速)
Comment[ja]=動的分割並列高速ダウンロード加速システム (64並列マルチプレックス)
Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 $BIN_DIR/dpls-gui %u
Icon=dpls-fast
Terminal=false
Categories=Network;FileTransfer;
MimeType=x-scheme-handler/dpls;
StartupNotify=true
"

echo "$DESKTOP_FILE_CONTENT" > "$APPS_DIR/dpls-fast.desktop"
chmod +x "$APPS_DIR/dpls-fast.desktop"

# 5. Create Desktop Shortcut
if [ -d "$DESKTOP_DIR" ]; then
  echo "🚀 正在建立桌面捷徑至 $DESKTOP_DIR/dpls-fast.desktop ..."
  echo "$DESKTOP_FILE_CONTENT" > "$DESKTOP_DIR/dpls-fast.desktop"
  chmod +x "$DESKTOP_DIR/dpls-fast.desktop"
  if command -v gio >/dev/null 2>&1; then
    gio set "$DESKTOP_DIR/dpls-fast.desktop" metadata::trusted true 2>/dev/null || true
  fi
fi

# 6. Configure Autostart on Boot
echo "⚡ 正在設定開機自動啟動 (Autostart on Boot)..."
mkdir -p "$AUTOSTART_DIR"
AUTOSTART_CONTENT="[Desktop Entry]
Version=1.0
Type=Application
Name=DPLS-Fast
Comment=DPLS-Fast Background Daemon
Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 $BIN_DIR/dpls-gui
Icon=dpls-fast
Terminal=false
Categories=Network;FileTransfer;
StartupNotify=false
X-GNOME-Autostart-enabled=true
"
echo "$AUTOSTART_CONTENT" > "$AUTOSTART_DIR/dpls-fast.desktop"

# 7. Deploy Browser Extension to dedicated local storage
EXTENSION_DIR="$HOME/.local/share/dpls-fast/extension"
if [ -d "$SCRIPT_DIR/extension" ]; then
  echo "🧩 正在部署瀏覽器擴充套件至 $EXTENSION_DIR ..."
  mkdir -p "$EXTENSION_DIR"
  cp -rf "$SCRIPT_DIR/extension/"* "$EXTENSION_DIR/"
fi

# 8. Update Desktop and Icon Caches
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo -e "\033[1;32m🎉 DPLS-Fast 安裝完成！\033[0m"
echo -e "  🔹 執行檔位置: \033[1;37m$BIN_DIR/dpls-gui\033[0m 與 \033[1;37m$BIN_DIR/dpls\033[0m"
echo -e "  🔹 應用程式選單: \033[1;37m$APPS_DIR/dpls-fast.desktop\033[0m"
if [ -d "$DESKTOP_DIR" ]; then
  echo -e "  🔹 桌面快捷圖示: \033[1;37m$DESKTOP_DIR/dpls-fast.desktop\033[0m"
fi
echo -e "  🔹 開機自動啟動: \033[1;37m$AUTOSTART_DIR/dpls-fast.desktop\033[0m (已啟用)"
echo -e "  🔹 瀏覽器擴充套件目錄: \033[1;37m$EXTENSION_DIR\033[0m"
echo ""
echo -e "\033[1;36m======================================================================\033[0m"
echo -e "\033[1;33m📌 瀏覽器擴充功能啟用說明（為什麼不安裝時靜默注入？）：\033[0m"
echo -e "  現代 Chromium 瀏覽器（Chrome / Edge / Brave）為了系統安全，"
echo -e "  全面禁止外部程式「靜默強制安裝」未在 Web Store 上架的擴充功能（防止流氓軟體偷裝）。"
echo -e ""
echo -e "  只需簡單一次性啟用步驟（花費 5 秒）："
echo -e "  1. 開啟 Chrome / Edge / Brave，網址列輸入: \033[1;34mchrome://extensions\033[0m"
echo -e "  2. 開啟右上角「\033[1;37m開發人員模式 (Developer Mode)\033[0m」開關"
echo -e "  3. 點擊「\033[1;37m載入未封裝項目 (Load unpacked)\033[0m」"
echo -e "  4. 選擇資料夾: \033[1;32m$EXTENSION_DIR\033[0m 即可完成！"
echo -e "\033[1;36m======================================================================\033[0m"
echo ""
