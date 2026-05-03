# Ddonilang (ddn) VS Code Shell

This folder contains the current-line VS Code shell for `.ddn` files.

## Install (local)
1. Open VS Code.
2. Run "Developer: Install Extension from Location...".
3. Select the folder: tools/vscode-ddn

## Current-Line Surface
- Syntax highlighting for `.ddn`.
- Starter snippets for `설정 {}`, `(시작)할때`, `(매마디)마다`, `보임 {}` 구조화 관찰, 조사/핀 예시.
- Fix-it rail stays in Seamgrim studio autofix. This package does not run semantic rewrite locally.

## Immediate Dev Priority
- 1순위: quick-fix
- 2순위: hook template completion
- 3순위: 조사/핀 helper
- ghost text: opt-in draft

## 지금 되는 것 / candidate-ready helper / 다음 것 / 아직 안 되는 것
- 지금 되는 것:
  - 설정/훅/snippet shell
  - `보임 {}` 구조화 관찰 snippet
  - 조사/핀 예시 snippet
  - `값~조사` / `값:핀` helper snippet
- candidate-ready helper:
  - `ddn-josa-helper-object`
  - `ddn-josa-helper-subject`
  - `ddn-pin-helper-fixed`
  - 현재 바로 쓸 수 있는 대표 최소선은 조사/핀 helper bundle이다.
- 다음 것:
  - parser/frontdoor diagnostic rail에 붙는 quick-fix
  - 조사/핀 helper를 diagnostic rail과 더 직접 잇는 안내
  - machine-readable prototype: `quickfix/ddn.quickfix-prototype.json`
- 안 되는 것:
  - semantic LSP
  - symbol index / hover / diagnostics server
  - full IDE workshop GUI

## 조사/핀 helper 선택 기준
- 먼저 `값~조사`를 쓴다.
  - 조사 role이 문맥상 자연스럽고 모호하지 않을 때
- `값:핀`을 쓴다.
  - 조사 role이 모호하거나 특정 핀으로 고정해야 할 때
- `보임 {}` snippet은 구조화 관찰 template다.
- 조사/핀 helper는 호출 표면 helper다.
- quick-fix는 parser/frontdoor diagnostic rail에 붙을 다음 승격 후보다.
- 이 shell은 현재 snippet helper까지만 제공한다.
  - semantic auto-rewrite나 full quick-fix server는 아직 열지 않는다.

## Quick-Fix Prototype

- prototype file:
  - `tools/vscode-ddn/quickfix/ddn.quickfix-prototype.json`
- bundle kind:
  - `one-code/one-edit`
- bundle size:
  - `2`
- current prototype attaches one parser/frontdoor diagnostic code to one minimal fix-it candidate.
  - `E_PARSE_EXPECTED_RBRACE` -> insert `}`
  - `E_PARSE_EXPECTED_RPAREN` -> insert `)`
- one-code/one-edit shell:
  - before: `보임 {` block missing closing `}`
  - after: closing `}` inserted at diagnostic end
  - before: `(1 + 2.` missing closing `)`
  - after: closing `)` inserted at diagnostic end
- 이것은 semantic rewrite가 아니라 attach point proof용 machine-readable prototype이다.
- 지금 되는 것:
  - helper/snippet shell
- 이번 라운드까지 되는 것:
  - two-case one-code/one-edit quick-fix shell
- 아직 안 되는 것:
  - semantic LSP
  - diagnostics server
  - full IDE workshop GUI

## Attach Points
- quick-fix: parser/frontdoor diagnostic rail
- hook template completion: VS Code snippet shell
- 조사/핀 helper: frontdoor/canon helper note
- ghost text: non-default draft, not current-line shell

## Boundary
- No semantic LSP yet.
- No symbol index / hover / diagnostics server.
- No full IDE workshop GUI productionization.

## Notes
- Comments: // line, /* block */.
- Supports suffix forms: josa (~을), pin fixing (:대상), units (10@m), resources (@"path").
- Keywords follow docs/ssot/ssot/SSOT_LANG_v17.0.9.md.
