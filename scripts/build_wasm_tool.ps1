param(
    [ValidateSet("release", "debug")]
    [string]$Profile = "release",
    [ValidateSet("web", "bundler", "nodejs", "no-modules")]
    [string]$Target = "web",
    [bool]$CopyToUi = $true
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

$buildRoot = "I:\\home\\urihanl\\ddn\\codex\\build"
if (-not (Test-Path $buildRoot)) {
    New-Item -ItemType Directory -Path $buildRoot -Force | Out-Null
}
if (-not (Test-Path $buildRoot)) {
    $buildRoot = "C:\\ddn\\codex\\build"
    New-Item -ItemType Directory -Path $buildRoot -Force | Out-Null
}

$outDir = Join-Path $buildRoot "wasm\\ddonirang_tool"
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

$wasmPack = Get-Command wasm-pack -ErrorAction SilentlyContinue
if ($wasmPack) {
    Push-Location (Join-Path $repoRoot "tool")
    $profileFlag = if ($Profile -eq "release") { "--release" } else { "" }
    & $wasmPack.Path build . --target $Target $profileFlag --out-dir $outDir --out-name ddonirang_tool -- --features wasm
    Pop-Location
    if ($CopyToUi) {
        & (Join-Path $repoRoot "scripts\\copy_wasm_tool_to_ui.ps1") -SourceRoot $outDir
    }
    Write-Host "WASM build complete (wasm-pack): $outDir"
    exit 0
}

$wasmBindgen = Get-Command wasm-bindgen -ErrorAction SilentlyContinue
if (-not $wasmBindgen) {
    Write-Error "wasm-pack or wasm-bindgen not found in PATH."
    exit 1
}

$profileFlag = if ($Profile -eq "release") { "--release" } else { "" }
& cargo build -p ddonirang-tool --target wasm32-unknown-unknown --features wasm $profileFlag

$wasmIn = Join-Path $repoRoot ("target\\wasm32-unknown-unknown\\{0}\\ddonirang_tool.wasm" -f $Profile)
if (-not (Test-Path $wasmIn)) {
    Write-Error "WASM not found: $wasmIn"
    exit 1
}

& $wasmBindgen.Path $wasmIn --target $Target --out-dir $outDir --out-name ddonirang_tool --no-typescript
if ($CopyToUi) {
    & (Join-Path $repoRoot "scripts\\copy_wasm_tool_to_ui.ps1") -SourceRoot $outDir
}
Write-Host "WASM build complete (wasm-bindgen): $outDir"
