@echo off
set NODEJS=node.exe
set "WORK_DIR=%~dp0"
set mainjs=node-chrome-pak.js
set file=chrome_100_percent.pak
set unpack-dir=unpacked
set modified-dir=modified
if not exist "%WORK_DIR%" md "%WORK_DIR%"
pushd "%WORK_DIR%"
if exist settings.ini for /f "tokens=* eol=; delims=" %%i in (settings.ini) do set %%i
if not exist "%WORK_DIR%\%modified-dir%" md "%WORK_DIR%\%modified-dir%" &exit
if exist "%WORK_DIR%\%file%" for %%i in (%modified-dir%\*) do %NODEJS% %mainjs% replace "%WORK_DIR%\%file%" %%~ni "%%~fsnxi"
