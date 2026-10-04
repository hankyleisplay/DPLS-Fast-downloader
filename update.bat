@echo off
chcp 65001 >nul
title DPLS-Fast Windows Updater
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\windows-updater.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo Update encountered an issue. Press any key to exit...
    pause >nul
)
