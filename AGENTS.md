# AGENTS.md

## 목적
이 문서는 Codex(또는 다른 에이전트)가 프로젝트에서 일할 때 따라야 할 규칙과 기대치를 정의한다.

## 작업 범위
- 이 저장소는 또니랑(DDN) 관문 0 구현과 관련된 코드를 포함한다.
- 주요 레이어: core(엔진), lang(문법/정본화), tool(실행 도구/런타임).

## 프로젝트 구조 요약
- `core/`: 엔진 코어
- `lang/`: 문법/정본화(파서/캐논)
- `tool/`: 런타임/도구 구현(관문0 기준)
- `tools/teul-cli/`: CLI 구현(골든 테스트는 `tools/teul-cli/tests/golden/`)
- `pack/`: D-PACK 입력/샘플/검증 데이터
- `docs/ssot/`: SSOT 전용(본문은 `docs/ssot/ssot/`, Codex 직접 수정 금지)
- `docs/context/`: 컨텍스트(제안/보고/AI 규격/ALL 묶음)
- `docs/context/design/`: 설계 초안
- `docs/steps/`: 단계 기록(관문0는 `docs/steps/000/`)
- `docs/status/`: 진행 상태/변경 기록
- `docs/context/roadmap/`: 로드맵/기획 문서
- `docs/studio/`: 스튜디오(탐구실) 문서
- `docs/guides/`: 학습 가이드/예제 모음 (초심자 튜토리얼, 실행 가능한 예제)
- `publish/`: 외부 공개용 문서
- `build/`: 로컬 산출물(배포 대상 아님). I:/home/urihanl/ddn/codex/build 를 사용해야 함. 시작 시 경로가 없으면 생성한다. I:/home/urihanl/ddn/codex/build 이 존재하지 않으면 자동으로 C:/ddn/codex/build 을 사용
- `out/`: 로컬 산출물(배포 대상 아님). I:/home/urihanl/ddn/codex/out 을 사용해야 함. 시작 시 경로가 없으면 생성한다. I:/home/urihanl/ddn/codex/out 이 존재하지 않으면 자동으로 C:/ddn/codex/out 을 사용
- `.cargo/config.toml`: cargo target-dir 설정(원드라이브 동기화 회피)

## 빠른 탐색 시작점
- `docs/ssot/ssot/SSOT_INDEX_v20.3.1.md`: SSOT 목차
- `docs/steps/INDEX.md`: 단계별 진행 상태
- `docs/status/PROJECT_STATUS.md`: 최근 변경 요약
- `docs/status/CHANGELOG.md`: 변경 내역
- `docs/guides/LEARNING_RESOURCES_INDEX.md`: 학습 자료 색인

## 기본 규칙
- 텍스트 파일은 UTF-8(BOM 없음)으로 저장한다.
- 파일 변경 전 관련 문서를 먼저 확인한다.
- docs/ssot/ssot/의 SSOT 본문과 `docs/ssot/walks/gogae#/`, `docs/ssot/age/age#/`의 단계/걸음 문서는 작업 전 반드시 읽고 준수한다.
- SSOT 정본은 `docs/ssot/ssot/`, `docs/ssot/walks/gogae#/`, `docs/ssot/age/age#/`만 인정한다.
  - 우선순위: `docs/ssot/ssot/`(v20.3.1) > `docs/ssot/walks/gogae#/` + `docs/ssot/age/age#/`(v20.3.1). 충돌 시 상위 SSOT 기준으로 진행하고 기록한다.
  - 고개별 지침: `docs/ssot/walks/gogae#/`
  - AGE별 지침: `docs/ssot/age/age#/`
  - WALK별 지침: `docs/ssot/walks/gogae#/WALK##/` (SPEC/IMPL_GUIDE/GOLDEN_TESTS)
- 걸음별 기획안은 `docs/ssot/ssot/SSOT_ROADMAP_CATALOG_v20.3.1.md`의 JIT 원칙(다가오는 1~3걸음만) 기준으로 작성/확인한다.
- 로드맵은 **설명용 기준**으로만 사용하고, 실제 구현/검증은 `docs/ssot/walks/gogae#/WALK##/{SPEC,IMPL_GUIDE,GOLDEN_TESTS}.md`를 우선한다.
- 도메인별 가지/팩/샘플은 `docs/ssot/gaji/`, `docs/ssot/pack/`, `docs/ssot/samples/`에서 확인한다.
- 기존 동작을 깨는 변경은 문서에 이유를 남긴다.
- 실행 예제는 docs/EXAMPLES에 갱신한다.
- 제안 문서는 `docs/context/proposals/`에 통일한다.
- 보고서 문서는 `docs/reports/`에 통일한다.
- 보고서 추가 시 `docs/reports/impl/INDEX.md` 또는 `docs/reports/audit/INDEX.md`를 갱신한다.
- 단계별 기록은 `docs/steps/<단계3자리>/`의 `PLAN.md/RESULT.md/TESTS.md/WORKLOG.md/HANDOFF.md`에 정리한다.
- 관문 0 기록은 `docs/steps/000/`을 기본 위치로 사용한다.
- TODO는 각 단계 `PLAN.md`의 `## TODO` 섹션에 기록한다.
- SSOT 변경 필요가 보이면 `docs/notes/`에 제안으로 정리하고, 합의 후 `docs/decisions/SSOT_CHANGELOG.md`에 기록한다.
- SSOT 본문 수정은 사용자님이 반영한다.
- 각 단계 `RESULT.md`에는 작업 요약과 함께 결정/이유를 기록한다.
- 각 단계 `WORKLOG.md`에는 상세 작업 내역(결정 근거, 커맨드, 실패/이슈 포함)을 기록한다.
- GitHub 업로드는 코드와 `publish/` 문서만 대상으로 한다.
- `docs/`는 로컬 참고용으로 유지하고, 공개할 문서는 `publish/`에 따로 정리한다.
- 사용자 공개 바이너리는 GitHub Releases에 배포하고 저장소에는 포함하지 않는다.

## docs/ 폴더 수정 권한
- SSOT 관리자 영역: `docs/ssot/**` 전체(본문/지침/템플릿). Codex는 **직접 수정 금지**.
- 개발 세션 영역: `docs/**` 중 `docs/ssot/**`를 제외한 모든 경로. Codex가 **직접 수정/이동/삭제 가능**.
- 침범 시 중단 규칙: 작업 중 `docs/ssot/**` 수정이 필요하거나 경계가 불명확하면 **즉시 중단**하고 사용자 승인/지시를 요청한다.

## 승인 및 안전
- 파괴적 명령(삭제/초기화)은 사용자 승인 후 진행한다.
- 네트워크/외부 다운로드는 사전 승인 없이 진행하지 않는다.

## 문서 업데이트
- 큰 변경이 있으면 docs/status/PROJECT_STATUS.md, docs/status/CHANGELOG.md를 갱신한다.
- 언어/런타임 변경은 docs/status/LANG_STATUS.md에 반영한다.

## 코딩 스타일
- 명시적이고 짧은 함수 선호.
- 메시지는 한국어로 유지한다.

## 테스트
- 변경 후 최소한 관련 테스트 또는 수동 실행을 수행한다.
- 실패한 테스트는 원인/해결을 문서화한다.

## 연락/인수인계
- 작업 요약, 변경 파일, 다음 제안 사항을 마지막에 남긴다.


