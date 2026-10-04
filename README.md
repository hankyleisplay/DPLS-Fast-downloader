# ⚡ DPLS-Fast (動態多路分段極速傳輸系統)

> 媲美甚至超越 IDM（Internet Download Manager）的高鐵級多線程動態分段傳輸系統。採用 Rust + Tokio 異步引擎與 Tauri 2 原生架構打造，具備全平台支援、現代簡約 **Liquid Glass (流體毛玻璃)** 美學、全域多國語言切換（繁中/簡中/英文/日文）、置頂搶焦攔截視窗與一鍵系統安裝版（含桌面捷徑與開機自啟動）。

---

## 🌟 核心特色 (Core Features)

1. **Liquid Glass 極簡流體毛玻璃美學**
   - 採用現代極簡玻璃擬物風格（macOS Tahoe / VisionOS 美學），深邃半透明高斯模糊背景（`backdrop-filter: blur(28px) saturate(190%)`）搭配鏡面邊框高光（Specular Reflection）。
   - 炫彩液態流光進度條（Liquid Wave Progress Animation）與極簡微光按鈕，兼顧極致效能與高級視覺體驗。

2. **多國語言國際化（i18n Multi-Language System）**
   - 內建 4 種主流語言無縫即時切換：
     - **繁體中文 (zh-TW)**
     - **简体中文 (zh-CN)**
     - **English (en-US)**
     - **日本語 (ja-JP)**
   - 依據系統語系自動偵測，並支援視窗內一鍵隨選切換。

3. **智慧前台置頂搶焦（Foreground Always-on-Top Focus）**
   - 攔截下載時，視窗自動計算螢幕正中央座標彈出，並調用系統搶焦 API（`focused: true`, `drawAttention: true` 與 `window.focus()`），徹底解決彈窗躲在瀏覽器後方的困擾。

4. **一鍵安裝版（桌面捷徑與開機自動啟動）**
   - **Linux**：內建 `./install.sh` 與 `./uninstall.sh`，自動部署至 `~/.local/bin`、建立應用程式選單、桌面快捷方式（`.desktop`）與開機自動常駐（`~/.config/autostart/`）。
   - **Windows**：提供 `scripts/windows-installer.ps1` 自動建立開始功能表、桌面捷徑與註冊 HKCU 開機自啟動登錄機碼。
   - **動態控制**：後端提供 `GET/POST /api/settings/autostart` 端點，隨時一鍵開關自啟動。

5. **IDM 級動態多路分段加速（Dynamic Multiplexed Slicing）**
   - 即時監控傳輸進度：當某線程率先完成時，自動動態切分剩餘最大區段並由閒置線程接力，保證 **64 併發連線全程 100% 頻寬滿載**。

6. **零合併磁區直接寫入（In-Place Zero-Merge Direct I/O）**
   - 開始傳輸時預分配磁碟區段，所有線程直接按 byte offset 寫入目標檔案。
   - 傳輸完成瞬間即可開啟，**完全無需等待臨時檔案合併**，保護磁碟壽命並節省時間。

7. **專業同名衝突策略與 SHA-256 自動校驗**
   - 支援「自動遞增重命名 (`file (1).ext`)」與「覆蓋現有檔案」策略。
   - 內建 SHA-256 完整性雜湊計算，支援與預期校驗碼自動對比。

---

## 📦 發行套件一覽 (Release Packages)

發行套件存放於 `dist/` 目錄：
- `dist/dpls-fast-windows-x86_64.zip`：Windows 完整安裝包（含執行檔、`install.bat`、擴充功能與開機自啟）
- `dist/dpls-fast-linux-x86_64.tar.gz`：Linux 完整安裝包（含 `dpls`、`dpls-gui`、`install.sh`、`uninstall.sh`）
- `dist/dpls-fast-extension-v1.3.zip`：瀏覽器擴充功能（Chrome / Edge / Brave / 支援 Chromium 系列）
- `dist/install.bat` / `dist/windows-installer.ps1`：Windows 一鍵自動化安裝腳本
- `dist/windows-uninstaller.ps1`：Windows 完整移除腳本

---

## 🚀 快速安裝與使用

### 🐧 Linux 一鍵安裝

```bash
# 解壓縮發行包或於專案目錄直接執行：
./install.sh
```
- 自動安裝至 `~/.local/bin/dpls-gui` 與 `~/.local/bin/dpls`
- 建立應用程式選單項目與桌面快捷圖示
- 註冊 `~/.config/autostart/dpls-fast.desktop` 實現開機自動啟動常駐

若需進行系統熱更新（更新執行檔、擴充套件並自動重啟背景服務）：
```bash
./update.sh
# 或重新編譯並更新
./update.sh --build
```

若需移除，只需執行：
```bash
./uninstall.sh
```

---

### 🪟 Windows 一鍵安裝與更新

**最簡單方式**：直接雙擊專案目錄下的 `install.bat`（或以 PowerShell 執行）：
```powershell
# 專案目錄或發行包目錄直接執行：
.\install.bat
# 或使用 PowerShell：
powershell -ExecutionPolicy Bypass -File install.ps1
```
- 自動安裝至 `%LOCALAPPDATA%\Programs\DPLSFast`
- 自動配置使用者 `PATH` 環境變數，可在終端隨處使用 `dpls` 命令
- 自動建立開始功能表與桌面快捷圖示
- 註冊開機自啟動登錄機碼 (`HKCU:\...\Run`)
- 自動啟動背景常駐服務

若需執行系統熱更新：
```powershell
.\update.bat
# 或
powershell -ExecutionPolicy Bypass -File update.ps1
```

若需移除：
```powershell
.\uninstall.bat
# 或
powershell -ExecutionPolicy Bypass -File uninstall.ps1
```

---

### 🧩 瀏覽器擴充功能安裝 (Chrome / Edge / Brave)

1. 瀏覽器網址列開啟 `chrome://extensions`（Edge 為 `edge://extensions`）。
2. 開啟右上角的「**開發人員模式**」（Developer Mode）。
3. 點擊「**載入未封裝項目**」（Load unpacked）。
4. 選擇本專案的 `extension/` 資料夾（或解壓 `dist/dpls-fast-extension-v1.2.zip`）。
5. 安裝完成！
   - 點擊網頁下載連結時自動彈出置頂 Liquid Glass 視窗。
   - 右上角隨時切換 繁中 / 簡中 / English / 日本語。
   - 支援「🚀 開始傳輸」直接在該視窗即時查看分段下載，或「⏳ 排入待傳佇列」。

---

### 💻 終端命令列極速下載 (CLI)

```bash
# 基本下載（預設 16 連線）
dpls https://example.com/large-file.zip

# 啟用 64 併發連線狂暴極速模式，指定檔名與目錄
dpls https://example.com/large-file.zip -c 64 -o myfile.zip -d ~/Downloads

# 啟動 Web 視覺化儀表板
dpls ui

# 以無頭背景伺服器模式運行
dpls server -p 6800
```
