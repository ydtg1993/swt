@echo off
chcp 65001 >nul
title SWT2 - 声文通 v2

cd /d "%~dp0"

echo ============================================================
echo  SWT2 - 声文通 v2
echo  启动中...
echo ============================================================
echo.
echo  Tauri 将自动启动 Python 推理服务器 (pythonw, 无黑窗)
echo.

call npm run tauri dev

pause
