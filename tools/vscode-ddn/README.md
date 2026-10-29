# Ddonilang VS Code Shell

This folder is the single owner for the local Ddonilang VS Code language contribution.

## Install (local)
1. In this folder run `npm ci --offline` (installs the locked `vscode-languageclient`; no download when the npm cache has it).
2. Build the server and CLI: `cargo build --release -p ddonirang-tool --bin ddonirang-lsp --bin ddonilang`.
3. Open VS Code and run "Developer: Install Extension from Location...".
4. Select the folder: tools/vscode-ddn
5. Outside this repository, set `ddonilang.lsp.executable` and `ddonilang.cli.executable` to the two built files, or put them in a `server/` folder inside the extension folder.

## 첫 흐름: 열기 → 오류 확인 → 고치기 → 저장하고 실행
- `.ddn` 파일을 열면 편집 중인 내용을 `ddonilang 검사 <파일> --source-profile v1-core-v25`와 같은 공통 분석으로 확인해 문제 창과 밑줄로 보여 줍니다. 저장하지 않은 편집 내용 기준이며, 디스크 파일과 섞지 않습니다.
- 오류가 없다는 표시는 이 공통 분석의 결과일 뿐입니다. 실행 결과까지 보장하지 않습니다.
- `또니랑: 저장하고 실행` (Ctrl+F5, 편집기 제목줄 실행 단추, 상태 표시줄 클릭): 저장한 뒤 `ddonilang 실행 <파일> --source-profile v1-core-v25`를 돌리고 결과를 `또니랑 실행` 출력 창에 보입니다. `ddonilang.run.madi`로 `--마디수`를 넘길 수 있습니다.
- 상태 표시줄: `또니랑 검사`(동작 중), `또니랑 서버 중지됨`(눌러서 다시 시작), `진단 미지원`(.ddl, .옮김), `저장 전 문서`(이름 없는 새 문서).
- 정의 이동·참조 찾기·강조·이름 바꾸기(F2)는 한 문서 안에서 공통 이름 결속이 확정한 이름만 다룹니다: 모든 사용이 확정된 최상위 함수(선언·결과 자리·`…하기` 호출), 그리고 최종 결속된 매개변수·지역 이름(루트 값 포함). 확정되지 않으면 정의 이동·참조는 결과를 내지 않고, 이름 바꾸기는 이유를 보이고 바꾸지 않습니다. 같은 철자의 다른 결속을 함께 보이지 않습니다.
- 이름 바꾸기 거부: 같은 철자의 지역 이름·매개변수와 겹치는 최상위 함수, 인자 없는 `…하기` 문장 호출, 공통 결속이 아직 확정하지 않는 위치, 새 이름이 이미 문서에 있을 때. 매개변수는 공개 핀 이름이기도 해서 다른 열린 문서에 같은 철자가 있으면 거부합니다.
- 작품(여러 파일): 얽음 뿌리 파일(`… 얽음.`)을 편집기에 열어 두면, 그 뿌리가 선언한 원문은 작품 단위로 결속합니다(디스크 원문 + 열린 문서의 현재 판). 폴더나 파일 이름으로 뿌리를 찾지 않습니다. 가져온 이름(`힘.속도`의 `속도`)에서 정의 이동은 의존 파일의 정의로 갑니다. 참조·강조·이름 바꾸기는 모든 사용이 확정된 매개변수·지역(예: `채비` 카운터)만 다루고, 이름 바꾸기는 이 문서 안의 편집만 냅니다. 내보낸 이름처럼 사용을 다 확정하지 못한 이름, 다른 파일에 걸친 이름, 작품 안의 함수 이름, 잠긴 꾸러미(`vendor/…`) 파일, 같은 문서를 선언한 뿌리가 둘 이상 열린 경우는 거부합니다. 원문을 바꾸면 `ddn.lock`은 맞지 않게 되므로, 실행 전에 `ddonilang 작품잠금`으로 잠금을 다시 만드세요.
- 작품 파일의 오류 표시는 아직 그 파일 하나만 검사합니다. 그래서 가져온 별칭(`힘`)이 "정의되지 않은 이름"으로 보일 수 있습니다.
- 스니펫 6개가 `.ddn` 문서의 완성 목록에 나오는 것을 실제 VS Code(격리 프로필, 고정 확장 7b5724fb)에서 독립검토로 확인했습니다.
- 내보임 관찰 스니펫(`(x축: t, 값: 값) 내보임.`)은 넣은 이름을 정의하면 검사를 통과하지만, 지금 제품에서는 실행이 실패합니다. 공통부에서 고치는 중입니다.
- `또니랑: 언어 서버 다시 시작`으로 서버를 새로 띄웁니다. 서버가 죽으면 클라이언트가 몇 번 다시 띄우고, 그래도 안 되면 멈춘 상태를 표시합니다.
## Current Surface
- Language id: `ddonilang`.
- File extensions: `.ddn`, `.ddl`, `.옮김`.
- Syntax highlighting for the registered Ddonilang surfaces.
- Starter snippets for `설정 {}`, `(시작)할때`, `(매마디)마다`, `보임 {}` 구조화 관찰, 조사/핀 예시.
- Fix-it rail stays in Seamgrim studio autofix. This package does not run semantic rewrite locally.

## Local LSP client

The extension starts the existing local `ddonirang-lsp` stdio server for the registered document types:

- `ddonilang.lsp.executable`: exact executable override.
- `ddonilang.lsp.autoDiscover`: bounded local discovery toggle.

- `vscode-languageclient` is pinned to `8.1.0` in the extension-local lockfile.
- Explicit executable paths must be local regular files; directories, symlinks, and missing paths fail closed.
- Auto-discovery checks only the extension's own `server/` folder, workspace `target/debug` and `target/release`, plus the repository-local target directories relative to this extension. The run command looks for `ddonilang` beside the server first.
- No PATH-wide search, remote server, source upload, or private dependency copy is used.

If no executable is available, the extension shows one clear error and does not retry indefinitely. Deactivation stops the client and its local server process.

## Immediate Dev Priority
- 1순위: quick-fix
- 2순위: hook template completion
- 3순위: 조사/핀 helper
- ghost text: opt-in draft

## 지금 되는 것 / candidate-ready helper / 다음 것 / 아직 안 되는 것
- 지금 되는 것:
  - 설정/훅/snippet shell
  - local stdio LSP client activation
  - local diagnostics/completion/navigation transport wiring
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
- remote or workspace-crawl semantic indexing
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
- The client consumes the existing local semantic LSP server; it does not duplicate parser or symbol ownership.
- No remote server, source upload, or filesystem crawl.
- No full IDE workshop GUI productionization.

## Notes
- Comments: // line, /* block */.
- Supports suffix forms: josa (~을), pin fixing (:대상), units (10@m), resources (@"path").
- Keywords follow docs/ssot/ssot/SSOT_LANG_v17.0.9.md.
