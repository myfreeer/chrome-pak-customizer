@echo off
if exist "%SystemRoot%\SysWOW64\cmd.exe" set x64=1
set NODEJS_x64=node64.exe
set NODEJS_x86=node32.exe
set "WORK_DIR=%~dp0"
set mainjs=node-chrome-pak.js
set file=chrome_100_percent.pak
set unpack-dir=unpacked
set modified-dir=modified
if not exist "%WORK_DIR%" md "%WORK_DIR%"
pushd "%WORK_DIR%"
if exist settings.ini for /f "tokens=* eol=; delims=" %%i in (settings.ini) do set %%i
if "x64"=="1" ( set NODEJS=%NODEJS_x64% ) else (set NODEJS=%NODEJS_x86%)
if not exist %NODEJS% echo nodejs(%NODEJS%) NOT FOUND! &pause &exit
if not exist "%WORK_DIR%\%unpack-dir%" md "%WORK_DIR%\%unpack-dir%"
if not exist "%WORK_DIR%\%modified-dir%" md "%WORK_DIR%\%modified-dir%"
if exist "%WORK_DIR%\%file%" %NODEJS% %mainjs% unpack "%WORK_DIR%\%file%" "%WORK_DIR%\%unpack-dir%"
timeout /t 5||pause