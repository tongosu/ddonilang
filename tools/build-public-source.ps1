# Project-authored developer build helper; Apache-2.0.
param([switch]$Offline, [switch]$BuildNative)
$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $checkArgs = @('check','--workspace','--lib','--bins','--locked','--no-default-features','--features','ddonirang-tool/v2-shared-modules')
    if ($Offline) { $checkArgs += '--offline' }
    & cargo @checkArgs
    if ($LASTEXITCODE -ne 0) { throw 'Production source check failed.' }
    if ($BuildNative) {
        $buildArgs = @('build','-p','ddonirang-tool','--bins','--locked','--no-default-features','--features','v2-shared-modules')
        if ($Offline) { $buildArgs += '--offline' }
        & cargo @buildArgs
        if ($LASTEXITCODE -ne 0) { throw 'Developer Native build failed.' }
    }
} finally { Pop-Location }
