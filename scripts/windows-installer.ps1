# ==============================================================================
# DPLS-Fast Windows Automated Installer (Desktop Shortcuts, PATH & Autostart)
# ==============================================================================

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "  🚀 DPLS-Fast 極速多線程傳輸加速器 - Windows 旗艦安裝程式" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""

# Determine Script Directory safely across all PowerShell hosts
$ScriptDir = $PSScriptRoot
if (-not $ScriptDir -and $MyInvocation.MyCommand.Path) {
    $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if (-not $ScriptDir) {
    $ScriptDir = (Get-Location).Path
}

# Determine candidate project root directories
$RootDir = Split-Path -Parent $ScriptDir
$CandidateRoots = @($ScriptDir, $RootDir, (Get-Location).Path) | Select-Object -Unique

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\DPLSFast"

# 1. Stop any running processes to prevent file locking
Write-Host "🛑 檢查並暫時終止正在運行的 DPLS 行程以防止檔案鎖定..." -ForegroundColor Gray
$RunningProcs = Get-Process -Name "dpls", "dpls-gui", "dpls-desktop", "DPLS-Fast" -ErrorAction SilentlyContinue
if ($RunningProcs) {
    $RunningProcs | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 800
}

# 2. Prepare Target Directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host "📦 正在部署 DPLS-Fast 至: $InstallDir ..." -ForegroundColor Gray

# Helper to find file in candidate roots and subfolders
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

# 3. Locate or Acquire Binaries
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

# If no local binaries, attempt cargo build if cargo is available
if (-not $GuiSrc -and -not $SetupExe) {
    $CargoCmd = Get-Command "cargo" -ErrorAction SilentlyContinue
    if ($CargoCmd) {
        Write-Host "⚙️ 偵測到本機已安裝 Cargo，正在建置 DPLS-Fast 發行二進位檔..." -ForegroundColor Yellow
        $BuildRoot = $null
        foreach ($r in $CandidateRoots) {
            if (Test-Path (Join-Path $r "Cargo.toml")) {
                $BuildRoot = $r
                break
            }
        }
        if ($BuildRoot) {
            try {
                Push-Location $BuildRoot
                & cargo build --release --bin dpls
                if (Test-Path "src-tauri\Cargo.toml") {
                    Push-Location "src-tauri"
                    & cargo build --release
                    Pop-Location
                }
                Pop-Location
                $CliSrc = Find-CandidateFile @("dpls.exe")
                $GuiSrc = Find-CandidateFile @("dpls-gui.exe", "dpls-desktop.exe", "DPLS-Fast.exe")
            } catch {
                Write-Host "⚠️ Cargo 建置遇到狀況，將嘗試網路線上取得發行套件..." -ForegroundColor DarkYellow
            }
        }
    }
}

# If still no GUI binary, automatically fetch latest official release from GitHub
if (-not $GuiSrc -and -not $SetupExe) {
    Write-Host "🌐 正在從官方 GitHub Releases 取得最新 Windows 發行套件..." -ForegroundColor Cyan
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13
        $apiUrl = "https://api.github.com/repos/hankyleisplay/DPLS-Fast-downloader/releases/latest"
        $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "DPLS-Installer" } -TimeoutSec 15
        
        $setupAsset = $release.assets | Where-Object { $_.name -like "*x64-setup.exe" } | Select-Object -First 1
        if ($setupAsset) {
            $tempSetup = Join-Path $env:TEMP $setupAsset.name
            Write-Host "  ⬇️ 下載官方安裝程式: $($setupAsset.name) ..." -ForegroundColor Gray
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
        Write-Host "⚠️ 無法連線至 GitHub API: $($_.Exception.Message)" -ForegroundColor DarkYellow
    }
}

# If SetupExe found, execute silent extraction to target install dir
if ($SetupExe -and (Test-Path $SetupExe)) {
    Write-Host "📦 正在透過 NSIS 引擎解套裝入: $InstallDir ..." -ForegroundColor Gray
    Start-Process -FilePath $SetupExe -ArgumentList "/S", "/D=$InstallDir" -Wait
    Start-Sleep -Seconds 1
}

# Copy CLI binary if found
if ($CliSrc -and (Test-Path $CliSrc)) {
    Copy-Item -Path $CliSrc -Destination (Join-Path $InstallDir "dpls.exe") -Force
    Write-Host "  ✔ 已部署 CLI 程式: dpls.exe" -ForegroundColor Green
}

# Copy GUI binary if found
if ($GuiSrc -and (Test-Path $GuiSrc)) {
    Copy-Item -Path $GuiSrc -Destination (Join-Path $InstallDir "dpls-gui.exe") -Force
    Copy-Item -Path $GuiSrc -Destination (Join-Path $InstallDir "dpls-desktop.exe") -Force
    Write-Host "  ✔ 已部署 GUI 程式: dpls-gui.exe" -ForegroundColor Green
}

# Ensure dpls-gui.exe and dpls-desktop.exe mirror each other in InstallDir
$DesktopExeInInstall = Join-Path $InstallDir "dpls-desktop.exe"
$GuiExeInInstall = Join-Path $InstallDir "dpls-gui.exe"

if ((Test-Path $DesktopExeInInstall) -and !(Test-Path $GuiExeInInstall)) {
    Copy-Item -Path $DesktopExeInInstall -Destination $GuiExeInInstall -Force
}
if ((Test-Path $GuiExeInInstall) -and !(Test-Path $DesktopExeInInstall)) {
    Copy-Item -Path $GuiExeInInstall -Destination $DesktopExeInInstall -Force
}

$TargetExe = $GuiExeInInstall
if (!(Test-Path $TargetExe)) {
    $TargetExe = $DesktopExeInInstall
}

if (!(Test-Path $TargetExe)) {
    Write-Host ""
    Write-Host "❌ 錯誤: 找不到可執行的 DPLS-Fast 二進位檔案！" -ForegroundColor Red
    Write-Host "  請確認本機有網路連線以下載最新版本，或在專案目錄使用 'cargo build --release' 完成編譯後重試。" -ForegroundColor Yellow
    exit 1
}

# 4. Copy Icon
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

# 5. Deploy Native Launcher (DPLS-Fast.exe)
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
        if (Test-Path $IconDest) { $IconArg = "/win32icon:`"$IconDest`"" }
        & $CscExe /target:winexe /optimize+ /r:System.Windows.Forms.dll /r:System.Drawing.dll $IconArg "/out:$LauncherDest" $LauncherCs | Out-Null
        if (Test-Path $LauncherDest) {
            $compiled = $true
            Write-Host "  ✔ 已編譯原生啟動器: DPLS-Fast.exe" -ForegroundColor Green
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

# 6. Create Windows Shortcuts (Desktop & Start Menu)
$WshShell = New-Object -ComObject WScript.Shell

# Desktop Shortcut
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$DesktopShortcutPath = Join-Path $DesktopPath "DPLS-Fast.lnk"
Write-Host "🖥️ 正在建立桌面捷徑至: $DesktopShortcutPath ..." -ForegroundColor Gray
$Shortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
$Shortcut.TargetPath = $ShortcutTarget
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "DPLS-Fast 極速多線程動態分段傳輸系統 (64 併發連線加速)"
if (Test-Path $IconDest) {
    $Shortcut.IconLocation = $IconDest
}
$Shortcut.Save()

# Start Menu Shortcut
$StartMenuPath = [Environment]::GetFolderPath("Programs")
$StartMenuShortcutPath = Join-Path $StartMenuPath "DPLS-Fast.lnk"
Write-Host "📂 正在建立開始功能表捷徑至: $StartMenuShortcutPath ..." -ForegroundColor Gray
$SmShortcut = $WshShell.CreateShortcut($StartMenuShortcutPath)
$SmShortcut.TargetPath = $ShortcutTarget
$SmShortcut.WorkingDirectory = $InstallDir
$SmShortcut.Description = "DPLS-Fast 極速多線程動態分段傳輸系統"
if (Test-Path $IconDest) {
    $SmShortcut.IconLocation = $IconDest
}
$SmShortcut.Save()

# 7. Configure Autostart on Boot (HKCU Run Key)
Write-Host "⚡ 正在配置開機自動啟動 (HKCU Run)..." -ForegroundColor Gray
$RunKeyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$RunValue = "`"$TargetExe`""
Set-ItemProperty -Path $RunKeyPath -Name "DPLSFast" -Value $RunValue -Force

# 8. Add InstallDir to User Environment PATH
Write-Host "🌐 正在配置環境變數 (User PATH)..." -ForegroundColor Gray
try {
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $CurrentPath) { $CurrentPath = "" }
    $PathParts = $CurrentPath -split ';' | Where-Object { $_ -ne "" }
    if ($PathParts -notcontains $InstallDir) {
        $NewPath = ($PathParts + $InstallDir) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Host "  ✔ 已將 DPLS-Fast 加入使用者 PATH，可於終端直接輸入 'dpls' 指令" -ForegroundColor Green
    }
} catch {
    Write-Host "  ⚠️ 環境變數 PATH 更新提示: $($_.Exception.Message)" -ForegroundColor DarkYellow
}

# 8. Deploy Browser Extension (Fix folder nesting)
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
    Write-Host "🧩 正在部署瀏覽器擴充套件至: $ExtDest ..." -ForegroundColor Gray
    Copy-Item -Path "$ExtSrc\*" -Destination $ExtDest -Recurse -Force
} else {
    # If not found locally, try downloading extension zip from releases
    Write-Host "🧩 正在線上下載瀏覽器擴充套件..." -ForegroundColor Gray
    try {
        $extZipUrl = "https://github.com/hankyleisplay/DPLS-Fast-downloader/releases/download/v1.3.0/dpls-fast-extension-v1.3.zip"
        $tempExtZip = Join-Path $env:TEMP "dpls-extension.zip"
        if (Get-Command "curl.exe" -ErrorAction SilentlyContinue) {
            & curl.exe -L -s -o $tempExtZip $extZipUrl
        } else {
            Invoke-WebRequest -Uri $extZipUrl -OutFile $tempExtZip -UseBasicParsing
        }
        if (Test-Path $tempExtZip) {
            Expand-Archive -Path $tempExtZip -DestinationPath $ExtDest -Force
            Remove-Item $tempExtZip -Force -ErrorAction SilentlyContinue
        }
    } catch {
        Write-Host "  ⚠️ 擴充套件下載提示: $($_.Exception.Message)" -ForegroundColor DarkYellow
    }
}

# 9. Deploy Web UI Assets (web\index.html)
$WebDest = Join-Path $InstallDir "web"
if (!(Test-Path $WebDest)) { New-Item -ItemType Directory -Path $WebDest -Force | Out-Null }
$WebSrc = $null
foreach ($r in $CandidateRoots) {
    $p = Join-Path $r "web\index.html"
    if (Test-Path $p) { $WebSrc = $p; break }
}
if ($WebSrc) {
    Copy-Item -Path $WebSrc -Destination (Join-Path $WebDest "index.html") -Force
    Write-Host "  ✔ 已部署最新 Web 儀表板: web\index.html" -ForegroundColor Green
}

# 10. Start Application
Write-Host "🚀 正在啟動 DPLS-Fast..." -ForegroundColor Gray
try {
    if (Test-Path $LauncherDest) {
        Start-Process -FilePath $LauncherDest -WorkingDirectory $InstallDir
        Write-Host "  ✔ DPLS-Fast 視窗與服務已成功啟動！" -ForegroundColor Green
    } else {
        Start-Process -FilePath $TargetExe -WorkingDirectory $InstallDir
        Write-Host "  ✔ 服務已於背景成功啟動！" -ForegroundColor Green
    }
} catch {
    Write-Host "  ⚠️ 啟動提示: 可點擊桌面捷徑手動啟動 DPLS-Fast" -ForegroundColor DarkYellow
}

Write-Host ""
Write-Host "🎉 DPLS-Fast 安裝完成！" -ForegroundColor Green
Write-Host "  🔹 程式目錄: $InstallDir"
Write-Host "  🔹 啟動程式: $ShortcutTarget"
Write-Host "  🔹 桌面捷徑: $DesktopShortcutPath"
Write-Host "  🔹 開始功能表: $StartMenuShortcutPath"
Write-Host "  🔹 開機自啟動: 已啟用 (註冊於 HKCU:\...\Run)"
Write-Host "  🔹 擴充套件目錄: $ExtDest"
Write-Host ""
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "📌 瀏覽器擴充功能啟用說明（只需 5 秒一次性啟用）：" -ForegroundColor Yellow
Write-Host "  1. 開啟 Chrome / Edge / Brave，網址列輸入: chrome://extensions"
Write-Host "  2. 開啟右上角「開發人員模式 (Developer Mode)」開關"
Write-Host "  3. 點擊「載入未封裝項目 (Load unpacked)」"
Write-Host "  4. 選擇資料夾: $ExtDest 即可完成！"
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host ""
