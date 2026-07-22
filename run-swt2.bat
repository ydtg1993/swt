@echo off
chcp 65001 >nul
title SWT2 - 声文通 v2

:: ============================================================
::  SWT2 快捷启动脚本 (Tauri v2 + React + Vite + TypeScript)
:: ============================================================

cd /d "%~dp0"

:menu
cls
echo.
echo   ╔══════════════════════════════════════════════╗
echo   ║         SWT2 - 声文通 v2  开发脚本          ║
echo   ╚══════════════════════════════════════════════╝
echo.
echo   [1] 启动桌面应用 (Tauri Dev)
echo   [2] 启动前端预览 (Vite Dev - 浏览器)
echo   [3] 构建桌面应用 (Tauri Build)
echo   [4] 构建前端 (Vite Build)
echo   [5] 预览构建产物 (Vite Preview)
echo   [6] 安装/更新依赖 (npm install)
echo   [7] 类型检查 (tsc)
echo   [0] 退出
echo.
set /p choice="请输入选项 [0-7]: "

if "%choice%"=="1" goto tauri_dev
if "%choice%"=="2" goto vite_dev
if "%choice%"=="3" goto tauri_build
if "%choice%"=="4" goto vite_build
if "%choice%"=="5" goto vite_preview
if "%choice%"=="6" goto npm_install
if "%choice%"=="7" goto tsc_check
if "%choice%"=="0" goto end
goto menu

:tauri_dev
cls
echo ============================================================
echo  启动 Tauri 桌面应用 (开发模式)
echo ============================================================
echo.
call npm run tauri dev
pause
goto menu

:vite_dev
cls
echo ============================================================
echo  启动 Vite 前端开发服务器 (浏览器)
echo ============================================================
echo.
call npm run dev
pause
goto menu

:tauri_build
cls
echo ============================================================
echo  构建 Tauri 桌面应用 (生产打包)
echo ============================================================
echo.
call npm run tauri build
echo.
echo 构建完成! 产物在 src-tauri\target\release\
pause
goto menu

:vite_build
cls
echo ============================================================
echo  构建前端 (TypeScript + Vite)
echo ============================================================
echo.
call npm run build
echo.
echo 构建完成! 产物在 dist\
pause
goto menu

:vite_preview
cls
echo ============================================================
echo  预览前端构建产物
echo ============================================================
echo.
call npm run preview
pause
goto menu

:npm_install
cls
echo ============================================================
echo  安装/更新 npm 依赖
echo ============================================================
echo.
call npm install
echo.
echo 安装完成!
pause
goto menu

:tsc_check
cls
echo ============================================================
echo  TypeScript 类型检查
echo ============================================================
echo.
call npx tsc --noEmit
echo.
echo 检查完成!
pause
goto menu

:end
exit /b
