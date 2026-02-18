param(
    [string]$SourceRoot = "I:\\home\\urihanl\\ddn\\codex\\build\\wasm\\ddonirang_tool",
    [string]$TargetRoot = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $TargetRoot) {
    $TargetRoot = Join-Path $repoRoot "solutions\\seamgrim_ui_mvp\\ui\\wasm"
}

if (-not (Test-Path $SourceRoot)) {
    $fallback = "C:\\ddn\\codex\\build\\wasm\\ddonirang_tool"
    if (Test-Path $fallback) {
        $SourceRoot = $fallback
    } else {
        Write-Error "WASM 산출물 폴더가 없습니다: $SourceRoot"
        exit 1
    }
}

New-Item -ItemType Directory -Path $TargetRoot -Force | Out-Null

$files = @(
    "ddonirang_tool.js",
    "ddonirang_tool_bg.wasm",
    "ddonirang_tool.d.ts",
    "package.json"
)

foreach ($name in $files) {
    $src = Join-Path $SourceRoot $name
    if (-not (Test-Path $src)) {
        Write-Error "필수 파일 누락: $src"
        exit 1
    }
    Copy-Item -Path $src -Destination (Join-Path $TargetRoot $name) -Force
}

Write-Host "WASM 산출물 복사 완료: $TargetRoot"
