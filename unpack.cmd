@echo off
set NODEJS=node.exe
set "WORK_DIR=%~dp0"
set mainjs=node-chrome-pak.js
set file=chrome_100_percent.pak
if not exist "%WORK_DIR%" md "%WORK_DIR%"
pushd "%WORK_DIR%"
if not exist "%WORK_DIR%\unpacked" md "%WORK_DIR%\unpacked"
if not exist "%WORK_DIR%\modified" md "%WORK_DIR%\modified"
if exist "%WORK_DIR%\%file%" %NODEJS% %mainjs% unpack "%WORK_DIR%\%file%" "%WORK_DIR%\unpacked"
