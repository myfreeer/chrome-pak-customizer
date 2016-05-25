@echo off
if exist "%SystemRoot%\SysWOW64\cmd.exe" set x64=1
set NODEJS_x64=node64.exe
set NODEJS_x86=node32.exe
set leanify_x64=Leanify64.exe
set leanify_x86=Leanify32.exe
set "WORK_DIR=%~dp0"
set mainjs=node-chrome-pak.js
set file=chrome_100_percent.pak
set unpack-dir=unpacked
set modified-dir=modified
set leanify_enabled=1
if not exist "%WORK_DIR%" md "%WORK_DIR%"
pushd "%WORK_DIR%"
if exist settings.ini for /f "tokens=* eol=; delims=" %%i in (settings.ini) do set %%i
if "x64"=="1" ( set NODEJS=%NODEJS_x64% ) else (set NODEJS=%NODEJS_x86%)
if "x64"=="1" ( set leanify=%leanify_x64% ) else (set leanify=%leanify_x86%)
if not exist %NODEJS% echo nodejs(%NODEJS%) NOT FOUND! &pause &exit
if not exist "%WORK_DIR%\%modified-dir%" md "%WORK_DIR%\%modified-dir%" &exit
if %leanify_enabled%==1 if exist %leanify% %leanify% -q "%WORK_DIR%\%modified-dir%"
if exist "%WORK_DIR%\%file%" for %%i in (%modified-dir%\*) do %NODEJS% %mainjs% replace "%WORK_DIR%\%file%" %%~ni "%%~fsnxi"
timeout /t 5||pause