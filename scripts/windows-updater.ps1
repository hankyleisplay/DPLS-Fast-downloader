# ==============================================================================
# DPLS-Fast Windows Automated Updater (Hot Reload & In-Place Update)
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "  🔄 DPLS-Fast 極速多線程傳輸加速器 - 系統更新程式 (Windows)" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""

# Determine Script Directory safely
$ScriptDir = $PSScriptRoot
if (-not $ScriptDir -and $MyInvocation.MyCommand.Path) {
    $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if (-not $ScriptDir) {
    $ScriptDir = (Get-Location).Path
}

$RootDir = Split-Path -Parent $ScriptDir
$CandidateRoots = @($ScriptDir, $RootDir, (Get-Location).Path) | Select-Object -Unique

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DPLSFast"

# 1. Check running processes & gracefully stop
$WasRunning = $false
$RunningProc = Get-Process -Name "dpls", "dpls-gui", "dpls-desktop", "DPLS-Fast" -ErrorAction SilentlyContinue
if ($RunningProc) {
    $WasRunning = $true
    Write-Host "🛑 偵測到 DPLS-Fast 正在運行，正在暫時終止以解除檔案鎖定..." -ForegroundColor Yellow
    $RunningProc | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 800
}

# 2. Verify target directory
if (!(Test-Path $InstallDir)) {
    Write-Host "⚠️ 尚未偵測到現有安裝目錄，正在自動初始化: $InstallDir ..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

function Find-CandidateFile {
    param([string[]]$Filenames)
    foreach ($fn in $Filenames) {
        foreach ($r in $CandidateRoots) {
            $pathsToCheck = @(
                (Join-Path $r $fn),
                (Join-Path $r "target\release\$fn"),
                (Join-Path $r "src-tauri\target\release\$fn"),
                (Join-Path $r "dist\$fn"),
                (Join-Path $r "dist\dpls-fast-windows-x86_64\$fn")
            )
            foreach ($p in $pathsToCheck) {
                if (Test-Path $p) {
                    return $p
                }
            }
        }
    }
    return $null
}

# 3. Locate Binaries or Update from GitHub Releases
$CliSrc = Find-CandidateFile @("dpls.exe")
$GuiSrc = Find-CandidateFile @("dpls-gui.exe", "dpls-desktop.exe")
$SetupExe = $null

foreach ($r in $CandidateRoots) {
    $setupCandidates = Get-ChildItem -Path $r -Filter "*setup*.exe" -ErrorAction SilentlyContinue
    if ($setupCandidates) {
        $SetupExe = $setupCandidates[0].FullName
        break
    }
    $distSetup = Get-ChildItem -Path (Join-Path $r "dist") -Filter "*setup*.exe" -ErrorAction SilentlyContinue
    if ($distSetup) {
        $SetupExe = $distSetup[0].FullName
        break
    }
}

if (-not $GuiSrc -and -not $SetupExe) {
    Write-Host "🌐 正在從官方 GitHub Releases 檢查並下載最新 Windows 發行版..." -ForegroundColor Cyan
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13
        $apiUrl = "https://api.github.com/repos/hankyleisplay/DPLS-Fast-downloader/releases/latest"
        $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "DPLS-Updater" } -TimeoutSec 15
        
        $setupAsset = $release.assets | Where-Object { $_.name -like "*x64-setup.exe" } | Select-Object -First 1
        if ($setupAsset) {
            $tempSetup = Join-Path $env:TEMP $setupAsset.name
            Write-Host "  ⬇️ 下載最新安裝套件: $($setupAsset.name) ..." -ForegroundColor Gray
            if (Get-Command "curl.exe" -ErrorAction SilentlyContinue) {
                & curl.exe -L -s -o $tempSetup $setupAsset.browser_download_url
            } else {
                Invoke-WebRequest -Uri $setupAsset.browser_download_url -OutFile $tempSetup -UseBasicParsing
            }
            if (Test-Path $tempSetup) {
                $SetupExe = $tempSetup
            }
        }
    } catch {
        Write-Host "  ⚠️ 線上更新提示: $($_.Exception.Message)" -ForegroundColor DarkYellow
    }
}

if ($SetupExe -and (Test-Path $SetupExe)) {
    Write-Host "📦 正在覆蓋更新主程式二進位檔..." -ForegroundColor Gray
    Start-Process -FilePath $SetupExe -ArgumentList "/S", "/D=$InstallDir" -Wait
    Start-Sleep -Seconds 1
}

if ($CliSrc -and (Test-Path $CliSrc)) {
    Copy-Item -Path $CliSrc -Destination (Join-Path $InstallDir "dpls.exe") -Force
    Write-Host "  ✔ 已更新 CLI: dpls.exe" -ForegroundColor Green
}

if ($GuiSrc -and (Test-Path $GuiSrc)) {
    Copy-Item -Path $GuiSrc -Destination (Join-Path $InstallDir "dpls-gui.exe") -Force
    Copy-Item -Path $GuiSrc -Destination (Join-Path $InstallDir "dpls-desktop.exe") -Force
    Write-Host "  ✔ 已更新 GUI: dpls-gui.exe" -ForegroundColor Green
}

# Ensure mirrors
$DesktopExeInInstall = Join-Path $InstallDir "dpls-desktop.exe"
$GuiExeInInstall = Join-Path $InstallDir "dpls-gui.exe"

if ((Test-Path $DesktopExeInInstall) -and !(Test-Path $GuiExeInInstall)) {
    Copy-Item -Path $DesktopExeInInstall -Destination $GuiExeInInstall -Force
}
if ((Test-Path $GuiExeInInstall) -and !(Test-Path $DesktopExeInInstall)) {
    Copy-Item -Path $GuiExeInInstall -Destination $DesktopExeInInstall -Force
}

$TargetExe = $GuiExeInInstall
if (!(Test-Path $TargetExe)) { $TargetExe = $DesktopExeInInstall }

# Deploy / Update Native Launcher (DPLS-Fast.exe)
$LauncherDest = Join-Path $InstallDir "DPLS-Fast.exe"
$LauncherCs = $null
foreach ($r in $CandidateRoots) {
    $p = Join-Path $r "scripts\launcher.cs"
    if (Test-Path $p) { $LauncherCs = $p; break }
}
$CscExe = "C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if (!(Test-Path $CscExe)) {
    $CscExe = "C:\Windows\Microsoft.NET\Framework\v4.0.30319\csc.exe"
}
$compiled = $false
if ($LauncherCs -and (Test-Path $CscExe)) {
    try {
        $IconArg = ""
        if (Test-Path (Join-Path $InstallDir "icon.ico")) { $IconArg = "/win32icon:`"$InstallDir\icon.ico`"" }
        & $CscExe /target:winexe /optimize+ /r:System.Windows.Forms.dll /r:System.Drawing.dll $IconArg "/out:$LauncherDest" $LauncherCs | Out-Null
        if (Test-Path $LauncherDest) {
            $compiled = $true
            Write-Host "  ✔ 已更新原生啟動器: DPLS-Fast.exe" -ForegroundColor Green
        }
    } catch {}
}

if (-not $compiled) {
    $LauncherSrc = Find-CandidateFile @("DPLS-Fast.exe")
    if ($LauncherSrc -and (Test-Path $LauncherSrc)) {
        Copy-Item -Path $LauncherSrc -Destination $LauncherDest -Force
        Write-Host "  ✔ 已部署啟動器: DPLS-Fast.exe" -ForegroundColor Green
    }
}

$ShortcutTarget = $LauncherDest
if (!(Test-Path $ShortcutTarget)) {
    $ShortcutTarget = $TargetExe
}

# 4. Update Icon
$IconDest = Join-Path $InstallDir "icon.ico"
$IconSrc = $null
foreach ($r in $CandidateRoots) {
    $icoCandidates = @(
        (Join-Path $r "src-tauri\icons\icon.ico"),
        (Join-Path $r "extension\icons\icon128.png"),
        (Join-Path $r "src-tauri\icons\icon.png")
    )
    foreach ($p in $icoCandidates) {
        if (Test-Path $p) {
            $IconSrc = $p
            break
        }
    }
    if ($IconSrc) { break }
}
if ($IconSrc -and (Test-Path $IconSrc)) {
    Copy-Item -Path $IconSrc -Destination $IconDest -Force
}

# 5. Update Browser Extension
$ExtDest = Join-Path $InstallDir "extension"
if (!(Test-Path $ExtDest)) {
    New-Item -ItemType Directory -Path $ExtDest -Force | Out-Null
}
$staleNestedExt = Join-Path $ExtDest "extension"
if (Test-Path $staleNestedExt) {
    Remove-Item -Path $staleNestedExt -Recurse -Force -ErrorAction SilentlyContinue
}

$ExtSrc = $null
foreach ($r in $CandidateRoots) {
    $p = Join-Path $r "extension"
    if (Test-Path (Join-Path $p "manifest.json")) {
        $ExtSrc = $p
        break
    }
}

if ($ExtSrc) {
    Write-Host "🧩 正在更新瀏覽器擴充套件檔案至: $ExtDest ..." -ForegroundColor Gray
    Copy-Item -Path "$ExtSrc\*" -Destination $ExtDest -Recurse -Force
    Write-Host "  ✔ 擴充套件 HTML / CSS (Liquid Glass) / JS / i18n 已就緒" -ForegroundColor Green
}

# 6. Update Web UI Assets (web\index.html)
$WebDest = Join-Path $InstallDir "web"
if (!(Test-Path $WebDest)) { New-Item -ItemType Directory -Path $WebDest -Force | Out-Null }
$WebSrc = $null
foreach ($r in $CandidateRoots) {
    $p = Join-Path $r "web\index.html"
    if (Test-Path $p) { $WebSrc = $p; break }
}
if ($WebSrc) {
    Copy-Item -Path $WebSrc -Destination (Join-Path $WebDest "index.html") -Force
    Write-Host "  ✔ 已更新 Web 儀表板: web\index.html" -ForegroundColor Green
}

# 7. Refresh Shortcuts
if (Test-Path $ShortcutTarget) {
    $WshShell = New-Object -ComObject WScript.Shell
    $DesktopShortcutPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "DPLS-Fast.lnk"
    if (Test-Path $DesktopShortcutPath) {
        $Shortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
        $Shortcut.TargetPath = $ShortcutTarget
        $Shortcut.WorkingDirectory = $InstallDir
        if (Test-Path $IconDest) {
            $Shortcut.IconLocation = $IconDest
        }
        $Shortcut.Save()
    }
    
    $SmShortcutPath = Join-Path ([Environment]::GetFolderPath("Programs")) "DPLS-Fast.lnk"
    if (Test-Path $SmShortcutPath) {
        $SmShortcut = $WshShell.CreateShortcut($SmShortcutPath)
        $SmShortcut.TargetPath = $ShortcutTarget
        $SmShortcut.WorkingDirectory = $InstallDir
        if (Test-Path $IconDest) {
            $SmShortcut.IconLocation = $IconDest
        }
        $SmShortcut.Save()
    }
}

# 7. Restart Service if it was running or autostart
if ($WasRunning -and (Test-Path $TargetExe)) {
    Write-Host "🚀 正在重新啟動 DPLS-Fast 背景常駐服務..." -ForegroundColor Gray
    Start-Process -FilePath $TargetExe -WorkingDirectory $InstallDir
    Start-Sleep -Seconds 1
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
