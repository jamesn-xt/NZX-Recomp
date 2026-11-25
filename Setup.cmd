@echo off
setlocal
powershell.exe -NoLogo -NoProfile -File "%~dp0Build.ps1" %*
if errorlevel 1 (
    echo.
    echo Setup stopped. See the message above.
    pause
    exit /b 1
)
