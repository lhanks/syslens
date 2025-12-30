@echo off
:: Syslens - Desktop System Information Dashboard
:: Run the production build

cd /d "%~dp0.."
start "" "projects\backend\target\release\syslens.exe"
