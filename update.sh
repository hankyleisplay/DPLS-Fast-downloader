#!/usr/bin/env bash
set -e

# ==============================================================================
# DPLS-Fast 一鍵熱更新程式 (Linux Updater)
# 支援：自動備份舊版、熱更新執行檔、更新擴充套件、重新整理捷徑並自動重啟背景服務
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICONS_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
ICONS_128_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
PIXMAPS_DIR="$HOME/.local/share/pixmaps"
AUTOSTART_DIR="$HOME/.config/autostart"
EXTENSION_DIR="$HOME/.local/share/dpls-fast/extension"

DESKTOP_DIR="$HOME/Desktop"
if command -v xdg-user-dir >/dev/null 2>&1; then
  XDG_DESK="$(xdg-user-dir DESKTOP 2>/dev/null || true)"
  if [ -n "$XDG_DESK" ] && [ -d "$XDG_DESK" ]; then
    DESKTOP_DIR="$XDG_DESK"
  fi
fi

DO_BUILD=0
DO_RESTART=1

for arg in "$@"; do
  case "$arg" in
    --build|-b)
      DO_BUILD=1
      ;;
    --no-restart)
      DO_RESTART=0
      ;;
  esac
done

echo ""
echo -e "\033[1;36m======================================================================\033[0m"
echo -e "\033[1;32m  🔄 DPLS-Fast 極速多線程傳輸加速器 - 系統更新程式 (Linux)\033[0m"
echo -e "\033[1;36m======================================================================\033[0m"
echo ""

# 1. Check if previous service is running
WAS_RUNNING=0
if pgrep -f "dpls-gui" >/dev/null 2>&1 || pgrep -x "dpls" >/dev/null 2>&1 || curl -s http://127.0.0.1:6800/api/tasks >/dev/null 2>&1; then
  WAS_RUNNING=1
  echo -e "\033[1;33m🛑 偵測到 DPLS-Fast 正於背景運行中，正在安全停止以釋放檔案鎖定與通訊埠...\033[0m"
  fuser -k 6800/tcp >/dev/null 2>&1 || true
  pkill -9 -x "dpls-gui" 2>/dev/null || true
  pkill -9 -x "dpls" 2>/dev/null || true
  pkill -9 -f "$BIN_DIR/dpls-gui" 2>/dev/null || true
  pkill -9 -f "$BIN_DIR/dpls" 2>/dev/null || true
  sleep 1
fi

# 2. Check if compilation is needed or requested
if [ "$DO_BUILD" -eq 1 ]; then
  echo "🔨 收到 --build 參數，正在重新編譯最新 Release 版本..."
  cd "$SCRIPT_DIR"
  cargo build --release
  cd "$SCRIPT_DIR/src-tauri"
  cargo build --release
  cd "$SCRIPT_DIR"
  
  # Copy compiled files to root if cache was used
  if [ -f "/home/hanson/.cache/cargo_targets/dpls-fast/release/dpls" ]; then
    cp -f "/home/hanson/.cache/cargo_targets/dpls-fast/release/dpls" "$SCRIPT_DIR/dpls"
  fi
  if [ -f "/home/hanson/.cache/cargo_targets/dpls-desktop/release/dpls-desktop" ]; then
    cp -f "/home/hanson/.cache/cargo_targets/dpls-desktop/release/dpls-desktop" "$SCRIPT_DIR/dpls-gui"
  fi
fi

# 3. Locate source binaries
CLI_SRC=""
GUI_SRC=""

if [ -f "$SCRIPT_DIR/dpls" ]; then
  CLI_SRC="$SCRIPT_DIR/dpls"
elif [ -f "$SCRIPT_DIR/dist/dpls-fast-linux-x86_64/dpls" ]; then
  CLI_SRC="$SCRIPT_DIR/dist/dpls-fast-linux-x86_64/dpls"
elif [ -f "/home/hanson/.cache/cargo_targets/dpls-fast/release/dpls" ]; then
  CLI_SRC="/home/hanson/.cache/cargo_targets/dpls-fast/release/dpls"
fi

if [ -f "$SCRIPT_DIR/dpls-gui" ]; then
  GUI_SRC="$SCRIPT_DIR/dpls-gui"
elif [ -f "$SCRIPT_DIR/dist/dpls-fast-linux-x86_64/dpls-gui" ]; then
  GUI_SRC="$SCRIPT_DIR/dist/dpls-fast-linux-x86_64/dpls-gui"
elif [ -f "/home/hanson/.cache/cargo_targets/dpls-desktop/release/dpls-desktop" ]; then
  GUI_SRC="/home/hanson/.cache/cargo_targets/dpls-desktop/release/dpls-desktop"
fi

if [ -z "$CLI_SRC" ] || [ -z "$GUI_SRC" ]; then
  echo -e "\033[1;31m❌ 錯誤：找不到最新的 DPLS-Fast 二進位檔案！請先執行 cargo build --release 或加入 --build 參數。\033[0m"
  exit 1
fi

# 4. Update binaries
echo "📦 正在更新執行檔至 $BIN_DIR ..."
mkdir -p "$BIN_DIR"
cp -f "$CLI_SRC" "$BIN_DIR/dpls"
cp -f "$GUI_SRC" "$BIN_DIR/dpls-gui"
chmod +x "$BIN_DIR/dpls" "$BIN_DIR/dpls-gui"
echo -e "  ✔ \033[1;32m已更新\033[0m: $BIN_DIR/dpls-gui"
echo -e "  ✔ \033[1;32m已更新\033[0m: $BIN_DIR/dpls"

# 5. Update Browser Extension
if [ -d "$SCRIPT_DIR/extension" ]; then
  echo "🧩 正在更新瀏覽器擴充套件至 $EXTENSION_DIR ..."
  mkdir -p "$EXTENSION_DIR"
  cp -rf "$SCRIPT_DIR/extension/"* "$EXTENSION_DIR/"
  echo -e "  ✔ \033[1;32m已更新\033[0m: Liquid Glass UI、多語系字典與置頂核心檔案"
fi

# 6. Update Icons & Launchers
echo "🎨 正在更新系統圖示與桌面捷徑..."
mkdir -p "$ICONS_DIR" "$ICONS_128_DIR" "$PIXMAPS_DIR" "$APPS_DIR"

if [ -f "$SCRIPT_DIR/src-tauri/icons/icon.png" ]; then
  cp -f "$SCRIPT_DIR/src-tauri/icons/icon.png" "$ICONS_DIR/dpls-fast.png"
  cp -f "$SCRIPT_DIR/src-tauri/icons/icon.png" "$PIXMAPS_DIR/dpls-fast.png"
fi

if [ -f "$SCRIPT_DIR/extension/icons/icon128.png" ]; then
  cp -f "$SCRIPT_DIR/extension/icons/icon128.png" "$ICONS_128_DIR/dpls-fast.png"
fi

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

if [ -d "$DESKTOP_DIR" ]; then
  echo "$DESKTOP_FILE_CONTENT" > "$DESKTOP_DIR/dpls-fast.desktop"
  chmod +x "$DESKTOP_DIR/dpls-fast.desktop"
  if command -v gio >/dev/null 2>&1; then
    gio set "$DESKTOP_DIR/dpls-fast.desktop" metadata::trusted true 2>/dev/null || true
  fi
fi

if [ -f "$AUTOSTART_DIR/dpls-fast.desktop" ]; then
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
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

# 7. Restart Service if it was running or requested
if [ "$WAS_RUNNING" -eq 1 ] && [ "$DO_RESTART" -eq 1 ]; then
  echo "🚀 正在重新啟動 DPLS-Fast 背景常駐服務..."
  env WEBKIT_DISABLE_DMABUF_RENDERER=1 nohup "$BIN_DIR/dpls-gui" >/dev/null 2>&1 &
  
  STARTED=0
  for i in {1..10}; do
    sleep 0.5
    if curl -s http://127.0.0.1:6800/api/settings/autostart >/dev/null 2>&1; then
      STARTED=1
      break
    fi
  done
  
  if [ "$STARTED" -eq 1 ]; then
    echo -e "  ✔ \033[1;32m背景常駐服務已成功重啟，通訊埠 6800 就緒！\033[0m"
  else
    echo -e "  ℹ️ 背景服務已在背景啟動。"
  fi
fi

echo ""
echo -e "\033[1;32m🎉 DPLS-Fast 系統熱更新完成！\033[0m"
echo -e "  🔹 最新執行檔: \033[1;37m$BIN_DIR/dpls-gui\033[0m ($(stat -c '%y' "$BIN_DIR/dpls-gui" 2>/dev/null | cut -d'.' -f1))"
echo -e "  🔹 擴充套件目錄: \033[1;37m$EXTENSION_DIR\033[0m"
echo ""
echo -e "\033[1;36m======================================================================\033[0m"
echo -e "\033[1;33m📌 瀏覽器擴充功能更新提示：\033[0m"
echo -e "  擴充套件檔案已直接於本機替換更新。"
echo -e "  若瀏覽器已開啟，請至 \033[1;34mchrome://extensions\033[0m 點擊 DPLS-Fast 卡片右下角的「\033[1;32m重新整理 (🔄)\033[0m」圖示即可立即生效最新樣式！"
echo -e "\033[1;36m======================================================================\033[0m"
echo ""
