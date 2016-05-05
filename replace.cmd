@echo off
set NODEJS=node.exe
set "WORK_DIR=%~dp0"
set mainjs=node-chrome-pak.js
set file=chrome_100_percent.pak
if not exist "%WORK_DIR%" md "%WORK_DIR%"
pushd "%WORK_DIR%"
if not exist "%WORK_DIR%\modified" md "%WORK_DIR%\modified" &exit
if exist "%WORK_DIR%\%file%"for %i in (unpacked\*) do %NODEJS% %mainjs% replace "%WORK_DIR%\%file%" %~ni "%~fsnxi"