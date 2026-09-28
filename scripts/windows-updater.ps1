# ==============================================================================
# DPLS-Fast Windows Automated Updater (Hot Reload & In-Place Update)
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "  🔄 DPLS-Fast 極速多線程傳輸加速器 - 系統更新程式 (Windows)" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RootDir = Split-Path -Parent $ScriptDir
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DPLSFast"

# 1. Check running processes & gracefully stop
$WasRunning = $false
$RunningProc = Get-Process -Name "dpls", "dpls-gui", "dpls-desktop" -ErrorAction SilentlyContinue
if ($RunningProc) {
    $WasRunning = $true
    Write-Host "🛑 偵測到 DPLS-Fast 正在運行，正在暫時終止以解除檔案鎖定..." -ForegroundColor Yellow
    $RunningProc | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

# 2. Verify target directory
if (!(Test-Path $InstallDir)) {
    Write-Host "⚠️ 尚未偵測到現有安裝目錄，正在自動初始化: $InstallDir ..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# 3. Update binaries
Write-Host "📦 正在覆蓋更新主程式二進位檔..." -ForegroundColor Gray
$FilesToCopy = @("dpls.exe", "dpls-gui.exe", "dpls-desktop.exe")
foreach ($f in $FilesToCopy) {
    $src = Join-Path $RootDir $f
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination $InstallDir -Force
        Write-Host "  ✔ 已更新: $f" -ForegroundColor Green
    }
}

$IconSrc = Join-Path $RootDir "src-tauri\icons\icon.ico"
if (Test-Path $IconSrc) {
    Copy-Item -Path $IconSrc -Destination (Join-Path $InstallDir "icon.ico") -Force
}

# 4. Update Browser Extension
$ExtSrc = Join-Path $RootDir "extension"
$ExtDest = Join-Path $InstallDir "extension"
if (Test-Path $ExtSrc) {
    Write-Host "🧩 正在更新瀏覽器擴充套件檔案至: $ExtDest ..." -ForegroundColor Gray
    Copy-Item -Path $ExtSrc -Destination $ExtDest -Recurse -Force
    Write-Host "  ✔ 擴充套件 HTML / CSS (Liquid Glass) / JS / i18n 已就緒" -ForegroundColor Green
}

# 5. Refresh Shortcuts
$WshShell = New-Object -ComObject WScript.Shell
$TargetExe = Join-Path $InstallDir "dpls-gui.exe"
if (!(Test-Path $TargetExe)) {
    $TargetExe = Join-Path $InstallDir "dpls-desktop.exe"
}

$DesktopShortcutPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "DPLS-Fast.lnk"
if (Test-Path $DesktopShortcutPath) {
    $Shortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $InstallDir
    if (Test-Path (Join-Path $InstallDir "icon.ico")) {
        $Shortcut.IconLocation = Join-Path $InstallDir "icon.ico"
    }
    $Shortcut.Save()
}

# 6. Restart Service if it was running
if ($WasRunning) {
    Write-Host "🚀 正在重新啟動 DPLS-Fast 背景常駐服務..." -ForegroundColor Gray
    Start-Process -FilePath $TargetExe
    Start-Sleep -Seconds 2
    Write-Host "  ✔ 背景常駐服務已成功重啟！" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 DPLS-Fast 系統熱更新完成！" -ForegroundColor Green
Write-Host "  🔹 程式目錄: $InstallDir"
Write-Host "  🔹 擴充套件目錄: $ExtDest"
Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "📌 瀏覽器擴充功能更新提示：" -ForegroundColor Yellow
Write-Host "  擴充套件檔案已直接覆蓋更新。"
Write-Host "  若瀏覽器已開啟，請至 chrome://extensions 點擊 DPLS-Fast 卡片右下角的「重新整理 (🔄)」圖示即可立即套用最新樣式！"
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""
