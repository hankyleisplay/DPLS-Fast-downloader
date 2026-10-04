@echo off
chcp 65001 >nul
title DPLS-Fast Windows Uninstaller
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\windows-uninstaller.ps1"
