# ==============================================================================
# DPLS-Fast Windows Automated Installer (Desktop Shortcuts & Autostart)
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "  🚀 DPLS-Fast 極速多線程傳輸加速器 - Windows 安裝程式" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RootDir = Split-Path -Parent $ScriptDir
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DPLSFast"

# 1. Prepare Target Directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host "📦 正在部署執行檔與資源至: $InstallDir ..." -ForegroundColor Gray

# 2. Copy binaries and icons if present
$FilesToCopy = @("dpls.exe", "dpls-gui.exe", "dpls-desktop.exe")
foreach ($f in $FilesToCopy) {
    $src = Join-Path $RootDir $f
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination $InstallDir -Force
        Write-Host "  ✔ 已複製: $f" -ForegroundColor Green
    }
}

$IconSrc = Join-Path $RootDir "src-tauri\icons\icon.ico"
if (Test-Path $IconSrc) {
    Copy-Item -Path $IconSrc -Destination (Join-Path $InstallDir "icon.ico") -Force
}

$TargetExe = Join-Path $InstallDir "dpls-gui.exe"
if (!(Test-Path $TargetExe)) {
    $TargetExe = Join-Path $InstallDir "dpls-desktop.exe"
}

# 3. Create Windows Shortcuts (Desktop & Start Menu)
$WshShell = New-Object -ComObject WScript.Shell

# Desktop Shortcut
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$DesktopShortcutPath = Join-Path $DesktopPath "DPLS-Fast.lnk"
Write-Host "🖥️ 正在建立桌面捷徑至: $DesktopShortcutPath ..." -ForegroundColor Gray
$Shortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
$Shortcut.TargetPath = $TargetExe
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "DPLS-Fast 極速多線程動態分段傳輸系統 (64 併發連線加速)"
if (Test-Path (Join-Path $InstallDir "icon.ico")) {
    $Shortcut.IconLocation = Join-Path $InstallDir "icon.ico"
}
$Shortcut.Save()

# Start Menu Shortcut
$StartMenuPath = [Environment]::GetFolderPath("Programs")
$StartMenuShortcutPath = Join-Path $StartMenuPath "DPLS-Fast.lnk"
Write-Host "📂 正在建立開始功能表捷徑至: $StartMenuShortcutPath ..." -ForegroundColor Gray
$SmShortcut = $WshShell.CreateShortcut($StartMenuShortcutPath)
$SmShortcut.TargetPath = $TargetExe
$SmShortcut.WorkingDirectory = $InstallDir
$SmShortcut.Description = "DPLS-Fast 極速多線程動態分段傳輸系統"
if (Test-Path (Join-Path $InstallDir "icon.ico")) {
    $SmShortcut.IconLocation = Join-Path $InstallDir "icon.ico"
}
$SmShortcut.Save()

# 4. Configure Autostart on Boot (HKCU Run Key)
Write-Host "⚡ 正在配置開機自動啟動 (HKCU Run)..." -ForegroundColor Gray
$RunKeyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$RunValue = "`"$TargetExe`""
Set-ItemProperty -Path $RunKeyPath -Name "DPLSFast" -Value $RunValue -Force

# 5. Deploy Extension
$ExtSrc = Join-Path $RootDir "extension"
$ExtDest = Join-Path $InstallDir "extension"
if (Test-Path $ExtSrc) {
    Write-Host "🧩 正在部署瀏覽器擴充套件至: $ExtDest ..." -ForegroundColor Gray
    Copy-Item -Path $ExtSrc -Destination $ExtDest -Recurse -Force
}

Write-Host ""
Write-Host "🎉 DPLS-Fast 安裝完成！" -ForegroundColor Green
Write-Host "  🔹 程式目錄: $InstallDir"
Write-Host "  🔹 桌面捷徑: $DesktopShortcutPath"
Write-Host "  🔹 開始功能表: $StartMenuShortcutPath"
Write-Host "  🔹 開機自動啟動: 已啟用 (註冊於 HKCU:\...\Run)"
Write-Host "  🔹 擴充套件目錄: $ExtDest"
Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "📌 瀏覽器擴充功能啟用說明（為什麼不安裝時靜默注入？）：" -ForegroundColor Yellow
Write-Host "  現代 Chromium 瀏覽器（Chrome / Edge / Brave）為了系統安全，"
Write-Host "  全面禁止外部程式「靜默強制安裝」未在 Web Store 上架的擴充功能（防止流氓軟體偷裝）。"
Write-Host ""
Write-Host "  只需簡單一次性啟用步驟（花費 5 秒）："
Write-Host "  1. 開啟 Chrome / Edge / Brave，網址列輸入: chrome://extensions"
Write-Host "  2. 開啟右上角「開發人員模式 (Developer Mode)」開關"
Write-Host "  3. 點擊「載入未封裝項目 (Load unpacked)」"
Write-Host "  4. 選擇資料夾: $ExtDest 即可完成！"
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""
