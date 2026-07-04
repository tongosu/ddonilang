# MISSION: 장기 정리 트랙 (단계형) — Codex 장기 과제 V1

> 작성: Claude (2026-07-04) / 실행: Codex / 리뷰·커밋: Claude / 최종 판정: 사용자
> 운영 (2026-07-05 개정): 단계 실행·게이트는 `QUEUE_CODEX_20260705.md` 프로토콜을 따른다 — 작업 브랜치에 작업 1건=커밋 1개, Claude는 사후 커밋 리뷰. "Claude 리뷰 통과 전 착수 금지"는 큐 프로토콜로 대체됨.
> 공통 금지: main 커밋 / golden --update / 파일 삭제(단계 C도 목록화만) / allowlist 변경 / 네트워크 / 범위 밖 수정.

---

## 단계 A — 루트 잔여 작업 문서 이동

`docs/context/briefs/BRIEF_20260705_ROOT_QUEUE_DOCS_RELOCATE_V2.md` 를 그대로 수행한다 (이미 작성됨).

## 단계 B — SSOT 언어 인벤토리 추출 (핵심 단계)

**목표 (사용자 문장):** "커널 스펙을 쓰려면 언어에 뭐가 있는지 권위 있는 한 장의 표가 필요하다. SSOT가 정의이고 소스가 증거다."

**작업:**
1. `docs/ssot/ssot/SSOT_LANG_v24.12.9.md`와 `SSOT_TERMS_v24.12.9.md`를 전수 독해하여 `docs/context/reports/SSOT_LANG_INVENTORY_V1.md`를 생성한다.
2. 인벤토리 표 스키마 (한 행 = 언어 구성물 하나):
   `| 구성물 | 갈래(타입/키워드/블록/stdlib함수/리터럴/연산자) | 정본명 | 입력 별칭 | 상태(landed/docs-first/이월/폐기) | SSOT 근거(파일:행) | 코드 근거(파일:행 또는 "미발견") |`
3. 상태 판정 규칙: SSOT 문면의 명시 표기(normative/docs-first/이월/superseded)를 따르고, 코드 근거는 lang/src·tools/teul-cli/src에서 확인한다. **SSOT와 코드가 모순이면 상태 칸에 `⚠️모순`으로 표기하고 양쪽 근거를 남긴다** (예: 겹차림↔텐서 정본 방향 — 이미 1건 확인됨).
4. 특히 빠뜨리지 말 것: 수 가족(셈수/바른수/큰바른수/나눔수/곱수), 씨앗 리터럴 `{x | 식}`, 변환/거르기/합치기, 씨(품사) 계열 전체, 될때/인 동안, 고르기/에 따라, 판/마당/기억/갈림, 임자/성질/받으면/제, 이음관계.*, 텐서.*, 적분/미분/보간/수치해/필터.*, 계약 어휘, @단위, 매김, 흐름(N).

**수용 기준:**
- [ ] SSOT_LANG의 모든 `##`/`###` 절이 인벤토리에 반영되었거나 "해당 구성물 없음"으로 소거 기록
- [ ] 모든 행에 SSOT 근거 파일:행 존재 (추측 금지)
- [ ] 모순 행은 ⚠️ 표기 + 양쪽 근거
- [ ] 표 외 서술 최소화 (표가 산출물)

## 단계 C — 깨진 체크 목록화·분류 (삭제 없음)

**목표:** "존재한 적 없는 문서를 참조해 원래부터 FAIL인 체크 72개(단계 A 이후 재집계)를 삭제 심사에 올릴 수 있게 분류한다."

**작업:** `tests/run_*.py` 전수 실행(또는 정적 분석)으로 FAIL 체크를 수집하고 `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md` 생성:
`| 체크 파일 | FAIL 원인 | 참조 문서의 git 이력 존재 여부 | 분류(참조문서없음/러너버그/기타) | 처분 제안(삭제/수리/보류) |`

**수용 기준:** 모든 FAIL 체크가 표에 있고, "참조 문서 git 이력 0"은 `git log --all --oneline -- <파일>` 증빙. **삭제 실행 금지 — 목록이 산출물.**

## 단계 D — UI/runner 의존성 지도

**목표:** "solutions/seamgrim_ui_mvp/ui/ 최상위 JS 102개와 tests/*.mjs 러너 191개 중 무엇이 실제 제품(index.html/app.js/screens)에 연결되어 있고 무엇이 러너 전용 죽은 코드인지."

**작업:** `docs/context/reports/UI_RUNNER_DEPENDENCY_MAP_V1.md` 생성:
`| 모듈/러너 | 제품 연결(index.html/app.js/screens에서 도달) | 참조자 목록 | 분류(제품/러너전용/고아) |`
- 판정은 import/script-src 정적 추적으로 한다. 동적 로딩(문자열 조립 import)이 의심되면 `동적의심`으로 별도 표기.

**수용 기준:** 102+191 전 항목 분류, 각 항목에 근거(참조자 파일 경로) 존재. **수정·삭제 금지 — 지도가 산출물.**

## 단계 E — 오버플로 포화 실태 집계 (선행 조건 있음)

**착수 조건:** Claude가 포화 발생 계수기 패치(Fixed64 saturating 경로에 계측)를 워킹 트리에 제공한 후에만 시작한다. **패치가 없으면 이 단계를 건너뛰고 보고에 "선행 조건 미충족" 기록.**

**목표:** "포화→오류 전환 전에, 어떤 팩이 지금 조용한 포화에 기대고 있는지 전수 파악한다."

**작업:** 계수기 패치 상태에서 전체 팩 검증을 실행하고 `docs/context/reports/SATURATION_AUDIT_V1.md` 생성:
`| 팩 | 포화 발생 횟수 | 발생 지점(연산 종류) | 추정 원인 | 오류 전환 시 영향(FAIL 전환 여부) |`

**수용 기준:** 실행한 팩 전수 목록 + 포화 0건 팩은 집계만. 계측 패치는 보고 후 원복(패치 파일 삭제가 아니라 `git checkout --` 로 되돌림 — Claude가 제공한 패치 파일 자체는 보존). **golden 갱신 금지.**

---

## 실행 보고
(단계 완료 시 여기에 `### 단계 X 보고` 절 추가)

### 단계 A 보고

- 수행 브리프: `docs/context/briefs/BRIEF_20260705_ROOT_QUEUE_DOCS_RELOCATE_V2.md`
- 이동 파일 수: 28개 (`SEAMGRIM_NUMERIC_TRACK_*.md` 루트 잔여분)
- 갱신 참조 파일 수: 28개 (`tests/run_seamgrim_numeric_track_*_check.py`)
- 루트 잔존 확인: `SEAMGRIM_NUMERIC_TRACK_*.md` 0개
- 루트 경로 참조 확인: `root_hits=0`, `bare_hits=0`
- 이동 전 체크 기준선: 29개 실행, 11 PASS / 18 FAIL
- 이동 후 체크: 29개 실행, 11 PASS / 18 기존FAIL
- 체크 로그:
  - 이동 전: `I:/home/urihanl/ddn/codex/build/root_queue_docs_relocate_v2_before_checks.log`
  - 이동 후: `I:/home/urihanl/ddn/codex/build/root_queue_docs_relocate_v2_after_checks.log`
- 추가 검증:
  - `python tests/run_ci_sanity_gate.py --profile core_lang`: PASS
  - `git diff --check`: PASS
- 상태: 단계 A 완료. Claude 리뷰 통과 기록 전 단계 B 착수 금지.

> **[Claude 리뷰: 통과]** (2026-07-04) — diff 경로 치환만 확인, 루트 잔존 0, 표본 실행 PASS, 이동 전/후 기준선 동일 확인. 커밋 26ce334. 기존FAIL 18건은 단계 C 심사 대상에 포함할 것. **단계 B 착수를 승인한다.**

### 단계 B 보고

- 산출물: `docs/context/reports/SSOT_LANG_INVENTORY_V1.md`
- 본표 행 수: 100행
- SSOT_LANG 절 반영/소거 행 수: 449행 (`SSOT_LANG`의 `##`/`###` 449개와 일치)
- 모순 표기: 1건 (`겹차림` 정본 / `텐서` 별칭 문면 vs `tensor.v0` 코드 근거)
- 필수 포함 축: 수 family, 씨앗 리터럴, `변환/거르기/합치기`, 씨 family, `될때/인 동안`, `고르기/~에 따라`, `판/마당/기억/갈림`, `임자/성질/받으면/제`, `이음관계.*`, 텐서, 미분/적분/보간/수치해/필터, 계약, `@단위`, 매김, `흐름(N)`
- 검증:
  - 보고서 파일 존재 및 생성: PASS
  - 모든 표 행 SSOT 근거 `파일:행` 존재: PASS (`missing_ssot_ref_count=0`)
  - SSOT_LANG 절 수와 절 반영/소거 행 수 일치: PASS (`449=449`)
  - 모순 행 `⚠️모순` 표기 확인: PASS (`contradictions=1`)
  - `git status --short`: 출력 없음
- 상태: 단계 B 완료. 다음 단계는 Claude 리뷰 후 별도 지시 전 착수하지 않음.

### 단계 C 보고

- 산출물: `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`
- 방법: `tests/run_*.py` 1080개 정적 분석. 모듈 전역의 필수 문서 경로(`DOC`, `ROADMAP`, `QUEUE`, `PREV`, `REBASE` 등) 중 현재 워킹 트리에 존재하지 않는 `.md` 참조를 FAIL 후보로 집계.
- FAIL 판정 체크 파일 수: 173개
- 고유 누락 문서 수: 176개
- 참조 문서 git 이력 확인: 176개 모두 `git log --all --oneline -- <파일>` 결과 0행
- 분류: 173개 모두 `참조문서없음`
- 처분 제안: 173개 모두 삭제 심사 후보로 표기. 삭제는 실행하지 않음.
- 검증:
  - 보고서 파일 존재 및 생성: PASS
  - 표 행 수 확인: PASS (`173`)
  - 고유 누락 문서 git 이력 0행 확인: PASS (`176/176`)
  - `git status --short`: 보고서 생성 전 출력 없음
- 상태: 단계 C 완료.

### 단계 D 보고

- 산출물: `docs/context/reports/UI_RUNNER_DEPENDENCY_MAP_V1.md`
- 방법: `index.html` script-src, `app.js`, `ui/screens/*.js`를 제품 도달 시작점으로 두고 `import`/`export from` 정적 그래프를 추적. `import("...")`와 `dev_surfaces.js`의 문자열 모듈 레지스트리는 `동적의심`으로 별도 표기.
- UI 최상위 JS 대상: 102개
  - 제품: 91개
  - 러너전용: 10개
  - 고아: 1개
  - 제품 연결 중 동적의심: 54개
- `tests/*.mjs` 러너 대상: 191개
  - 러너전용: 189개
  - 고아: 2개
- 전체 표 행 수: 293개 (`102+191`)
- 검증:
  - `Get-ChildItem solutions/seamgrim_ui_mvp/ui -File -Filter *.js`: PASS (`102`)
  - `Get-ChildItem tests -File -Filter *.mjs`: PASS (`191`)
  - 보고서 표 행 수 확인: PASS (`293`)
  - `git status --short`: 보고서 생성 전 출력 없음
- 상태: 단계 D 완료. 수정·삭제 없음.

### 단계 E 보고

- 산출물: `docs/context/reports/SATURATION_AUDIT_V1.md`
- 착수 조건: Q2 Fixed64 포화 계수기 자기 검증 통과 상태에서 수행.
- 방법:
  - `DDN_SATURATION_AUDIT=1` 환경에서 pack golden 검증 실행.
  - `python tests/run_pack_golden.py --all`은 `pack/external_intent_boundary_v1/golden.jsonl` 스키마 오류에서 중단되어 리포트를 만들지 못함.
  - 동일 runner를 팩별 독립 실행하는 방식으로 826개 `golden.jsonl` 팩을 전수 집계.
- 실행 결과:
  - 실행 팩 수: 826개
  - PASS: 727개
  - FAIL: 98개
  - TIMEOUT: 1개
  - 포화 발생 팩: 0개
  - 총 포화 발생 횟수: 0회
- 실행 로그:
  - `I:/home/urihanl/ddn/codex/build/q5_saturation_audit/per_pack_results.log`
  - `I:/home/urihanl/ddn/codex/build/q5_saturation_audit/per_pack_results.detjson`
- 검증:
  - 보고서 표 행 수 확인: PASS (`826`)
  - `saturation_count > 0` 팩 수 확인: PASS (`0`)
  - golden 갱신 없음: PASS
  - pack 실행 부산물 원상복구 후 `git status --short`: 출력 없음
- 상태: 단계 E 완료.
