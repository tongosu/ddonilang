param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$preferred = "I:/home/urihanl/ddn/codex/target"
$fallback = "C:/ddn/codex/target"
$targetDir = $preferred

if (-not (Test-Path $preferred)) {
    $targetDir = $fallback
}

New-Item -ItemType Directory -Force $targetDir | Out-Null
$env:CARGO_TARGET_DIR = $targetDir

cargo @Args
exit $LASTEXITCODE
