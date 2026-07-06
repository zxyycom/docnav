@echo off
setlocal
set "DOCNAV_ROOT=%~dp0"
bun "%DOCNAV_ROOT%scripts\docnav-local\run.ts" %*
exit /b %ERRORLEVEL%
