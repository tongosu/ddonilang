$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Publish = Join-Path $RepoRoot "publish"
$I18nRoot = Join-Path $Publish "i18n"
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)

$Locales = @(
  @{ code="ko"; name="Korean"; readme="README.md"; title="Ddonilang"; note="Canonical Korean public document set." },
  @{ code="en"; name="English"; readme="README_en.md"; title="Ddonilang"; note="English public guide." },
  @{ code="ja"; name="Japanese"; readme="README_ja.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="tr"; name="Turkish"; readme="README_tr.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="mn"; name="Mongolian"; readme="README_mn.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="ay"; name="Aymara"; readme="README_ay.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="eu"; name="Basque"; readme="README_eu.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="kn"; name="Kannada"; readme="README_kn.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="ne"; name="Nepali"; readme="README_ne.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="qu"; name="Quechua"; readme="README_qu.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="sym3"; name="sym3"; readme="README_sym3.md"; title="Ddonilang"; note="Compact symbolic public guide." },
  @{ code="ta"; name="Tamil"; readme="README_ta.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="te"; name="Telugu"; readme="README_te.md"; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="zh"; name="Chinese"; readme=$null; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="es"; name="Spanish"; readme=$null; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="fr"; name="French"; readme=$null; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." },
  @{ code="de"; name="German"; readme=$null; title="Ddonilang"; note="Starter localized guide; commands and file names stay canonical." }
)

function Write-Utf8NoBom($Path, $Text) {
  $Dir = Split-Path -Parent $Path
  if (-not (Test-Path $Dir)) {
    New-Item -ItemType Directory -Force -Path $Dir | Out-Null
  }
  [System.IO.File]::WriteAllText($Path, ($Text -replace "`r`n", "`n"), $Utf8NoBom)
}

function New-Readme($L) {
@"
# $($L.title) ($($L.code))

> $($L.note)

## Current status

- Korean-native DDN is the canonical language surface.
- Rust 'ddonirang-lang', 'ddonirang-tool', and 'teul-cli' validate syntax, runtime behavior, packs, and current-line execution.
- Seamgrim is a WASM-first web workspace for DDN examples, Bogae views, mirror records, and result tables.
- Bogae is a view layer. Runtime truth stays in DDN runtime, packs, state hashes, and mirror/replay records.

## Main documents

- Quick start: 'QUICKSTART.md'
- Development structure: 'DEV_STRUCTURE.md'
- Downloads: 'DOWNLOADS.md'
- Release notes: 'RELEASE_NOTES_20260211.md'
- Korean canonical public README: '../../README.md'
- English reference README: '../../README_en.md'

## Current examples

- console-grid minimal example
- space2d pendulum and bounce probe
- console-grid Tetris
- formula/proof/lambda examples
- maze probe

## Localization status

This document is a starter localization. Native review is still required before treating it as a polished public translation.
"@
}

function New-Quickstart($L) {
@"
# Quick start ($($L.name))

> $($L.note)

## 1. Build from source

Requirements: Rust + Cargo

~~~sh
cargo build --release
~~~

Check the CLI:

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Run Seamgrim workspace

Start the local server:

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Open:

~~~txt
http://localhost:8787/
~~~

The workspace can open examples from 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Product regression checks

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Binary release path

When release binaries are published, download them from GitHub Releases. Binaries are not stored in the git repository.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux: 'chmod +x ./ddonirang-tool' then './ddonirang-tool --help'
"@
}

function New-DevStructure($L) {
@"
# Development structure ($($L.name))

> $($L.note)

This is a localized public summary. The canonical detailed file is '../../DDONIRANG_DEV_STRUCTURE.md'.

## Core layers

| Layer | Path | Role |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | command-line execution and checks |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace and Bogae views |
| tests | 'tests/' | integration and product checks |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': single entry point
- 'ui/screens/run.js': run screen and current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': local static server and helper API

## Runtime principle

- DDN runtime, packs, state hashes, and mirror/replay records own truth.
- Bogae is a view layer and must not own runtime truth.
- Python/JS may orchestrate checks and UI, but they must not replace language/runtime semantics with test-only lowering.

## Current evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product stabilization smoke
- Bogae madi/graph UI checks
"@
}

function New-Downloads($L) {
@"
# Downloads ($($L.name))

> $($L.note)

## Distribution location

- Public binaries belong in GitHub Releases.
- The git repository does not store user-facing binaries.

## Target platforms

- Windows x64
- macOS x64/arm64
- Linux x64/arm64

## Recommended file names

- 'ddonirang-tool-<version>-windows-x64.zip'
- 'ddonirang-tool-<version>-macos-arm64.zip'
- 'ddonirang-tool-<version>-linux-x64.tar.gz'

## Recommended package layout

~~~txt
ddonirang-tool-<version>-<os>-<arch>/
  ddonirang-tool(.exe)
  LICENSE
  NOTICE.txt
  README.txt
~~~

## Checksums

Provide 'SHA256SUMS.txt' with releases. Add a signature when available.

## Source path

For current development, build from source and run Seamgrim locally. See 'QUICKSTART.md'.
"@
}

function New-ReleaseNotes($L) {
@"
# Release notes 2026-02-11 ($($L.name))

> Historical release note. Current public entry points are '../../README.md', '../../QUICKSTART.md', and '../../DDONIRANG_DEV_STRUCTURE.md'.

## Summary

This release focused on AGE2 Open policy hardening and minimum schemas/runtime APIs for open.net, open.ffi, and open.gpu.

## Highlights

- 'open=record|replay' is blocked when 'age_target < AGE2'.
- '--unsafe-open' was added as an explicit bypass.
- open log schemas were added:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- Packs were added:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Behavior change

'open=record|replay' is allowed only for 'age_target >= AGE2' unless '--unsafe-open' is used.

## Historical test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Current status pointer

For current Seamgrim/WASM/current-line status, use 'QUICKSTART.md' and 'DEV_STRUCTURE.md' in this language folder.
"@
}

if (-not (Test-Path $I18nRoot)) {
  New-Item -ItemType Directory -Force -Path $I18nRoot | Out-Null
}

$IndexRows = @()
foreach ($L in $Locales) {
  $Dir = Join-Path $I18nRoot $L.code
  New-Item -ItemType Directory -Force -Path $Dir | Out-Null

  Write-Utf8NoBom (Join-Path $Dir "README.md") (New-Readme $L)

  Write-Utf8NoBom (Join-Path $Dir "QUICKSTART.md") (New-Quickstart $L)
  Write-Utf8NoBom (Join-Path $Dir "DEV_STRUCTURE.md") (New-DevStructure $L)
  Write-Utf8NoBom (Join-Path $Dir "DOWNLOADS.md") (New-Downloads $L)
  Write-Utf8NoBom (Join-Path $Dir "RELEASE_NOTES_20260211.md") (New-ReleaseNotes $L)

  $IndexRows += "| '$($L.code)' | $($L.name) | 'publish/i18n/$($L.code)/README.md' | starter/localized |"
}

$Index = @"
# publish/i18n index

This folder contains localized public document sets. Command names, file paths, package names, and runtime contracts stay canonical across languages.

| Code | Name | Entry | Status |
| --- | --- | --- | --- |
$($IndexRows -join "`n")

## Included documents

Each language folder contains:

- 'README.md'
- 'QUICKSTART.md'
- 'DEV_STRUCTURE.md'
- 'DOWNLOADS.md'
- 'RELEASE_NOTES_20260211.md'

## Boundary

These files are public localization guides. They do not own DDN language/runtime truth.
"@
Write-Utf8NoBom (Join-Path $I18nRoot "INDEX.md") $Index
