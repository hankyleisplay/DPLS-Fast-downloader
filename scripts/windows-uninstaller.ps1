# ==============================================================================
# DPLS-Fast Windows Uninstaller
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "SilentlyContinue"

Write-Host ""
Write-Host "======================================================================" -ForegroundColor Magenta
Write-Host "  🗑️ DPLS-Fast 極速多線程傳輸加速器 - 移除程式 (Windows)" -ForegroundColor Yellow
Write-Host "======================================================================" -ForegroundColor Magenta
Write-Host ""

# 1. Kill running instances
Write-Host "🛑 正在終止背景行程..." -ForegroundColor Gray
$RunningProcs = Get-Process -Name "dpls", "dpls-gui", "dpls-desktop", "DPLS-Fast" -ErrorAction SilentlyContinue
if ($RunningProcs) {
    $RunningProcs | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 600
}

# 2. Remove Autostart Registry Key
Write-Host "⚡ 正在移除開機自啟登錄檔..." -ForegroundColor Gray
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "DPLSFast" -ErrorAction SilentlyContinue

# 3. Remove Shortcuts
Write-Host "🗑️ 正在移除桌面與開始功能表捷徑..." -ForegroundColor Gray
$DesktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "DPLS-Fast.lnk"
if (Test-Path $DesktopShortcut) { Remove-Item -Path $DesktopShortcut -Force -ErrorAction SilentlyContinue }

$SmShortcut = Join-Path ([Environment]::GetFolderPath("Programs")) "DPLS-Fast.lnk"
if (Test-Path $SmShortcut) { Remove-Item -Path $SmShortcut -Force -ErrorAction SilentlyContinue }

# 4. Remove InstallDir from User PATH
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DPLSFast"
try {
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath) {
        $PathParts = $CurrentPath -split ';' | Where-Object { $_ -ne "" -and $_ -ne $InstallDir }
        $NewPath = $PathParts -join ';'
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    }
} catch {}

# 5. Remove Files
if (Test-Path $InstallDir) {
    Write-Host "🗑️ 正在清除安裝目錄: $InstallDir ..." -ForegroundColor Gray
    Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "✅ DPLS-Fast 已完全從系統中解除安裝！" -ForegroundColor Green
Write-Host ""
