@echo off
setlocal

set "DDN_BIN=i:\home\urihanl\ddn\codex\target\debug\teul-cli.exe"
if exist "%DDN_BIN%" goto run

set "DDN_BIN=%~dp0tools\teul-cli\target\debug\teul-cli.exe"
if exist "%DDN_BIN%" goto run

set "DDN_BIN=teul-cli.exe"

:run
"%DDN_BIN%" %*
