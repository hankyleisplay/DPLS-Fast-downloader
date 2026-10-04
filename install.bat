@echo off
chcp 65001 >nul
title DPLS-Fast Windows Installer
echo ======================================================================
echo   DPLS-Fast Installer for Windows
echo ======================================================================
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\windows-installer.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo Installation encountered an issue. Press any key to exit...
    pause >nul
)
