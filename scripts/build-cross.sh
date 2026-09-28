#!/usr/bin/env bash
set -e

echo "🚀 DPLS-Fast 跨平台編譯工具"
echo "=================================="

# Check target
if [ "$1" == "windows" ]; then
    echo "🔨 準備編譯 Windows 版本 (x86_64-pc-windows-gnu)..."
    rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
    if which x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        cargo build --release --target x86_64-pc-windows-gnu
        echo "✅ Windows CLI 編譯成功：target/x86_64-pc-windows-gnu/release/dpls.exe"
    else
        echo "⚠️ 尚未安裝 mingw-w64。請在 Ubuntu 上執行: sudo apt install -y mingw-w64"
        echo "💡 或者直接推送 GitHub Tag (如 git tag v1.2.0 && git push --tags)，由 GitHub Actions 自動為您雲端編譯出官方 Windows (.exe) 與 macOS (.dmg) 安裝包！"
    fi
else
    echo "使用方式: ./scripts/build-cross.sh [windows]"
    echo "支援之多平台 CI 建置矩陣位於: .github/workflows/release.yml"
fi
