param(
    [switch]$Execute,
    [string]$Version = "v23.0.5"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$sourceDir = Join-Path $repoRoot "docs/ssot/ssot"
$targetDir = Join-Path $repoRoot ("docs/ssot/releases/" + $Version)

if (!(Test-Path $sourceDir)) {
    Write-Error "source directory not found: $sourceDir"
}

$versionCore = [regex]::Escape($Version.TrimStart('v'))
$files = Get-ChildItem -Path $sourceDir -File |
    Where-Object { $_.Name -match "_v$versionCore\.md$" }

Write-Host "policy=copy_only version=$Version"
Write-Host "source=$sourceDir"
Write-Host "target=$targetDir"
Write-Host "file_count=$($files.Count)"

foreach ($f in $files) {
    $dest = Join-Path $targetDir $f.Name
    Write-Host ("plan copy: " + $f.FullName + " -> " + $dest)
}

if (-not $Execute) {
    Write-Host "mode=dry_run (use -Execute to perform copy)"
    exit 0
}

if (!(Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

foreach ($f in $files) {
    $dest = Join-Path $targetDir $f.Name
    Copy-Item -LiteralPath $f.FullName -Destination $dest -Force
}

Write-Host "done=copy_only_no_delete"
