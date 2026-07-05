# BRIEF: 가지(gaji) 패키지 뼈대 실태 조사 — `gaji new` 설계 재료

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 성격: 조사·설계 재료 수집. 실제 CLI 명령 구현은 이 브리프 범위 아님(설계는 Claude가 함).
> 배경: `GajiCommands`(Lock/Install/Update/Vendor/Registry)에 새 패키지를 만드는 명령이 없음을 확인. 기존 30개 `gaji/` 패키지가 실제로 어떤 구조·필드를 쓰는지 조사해 향후 `gaji new` 설계의 재료로 삼는다.

## 작업

1. 저장소의 실제 `gaji/*` 디렉터리(30개) 전수를 훑어, 각 패키지가 공통으로 갖는 파일/디렉터리 구조를 표로 정리(예: `gaji.toml`, `README.md`, 소스 파일 배치 관례 등).
2. `gaji.toml` 실제 필드 사용 현황 전수 — `id`/`version` 외에 실제로 쓰이는 필드가 있는지(`[requires].det_tier`, `[requires].openness` 등 SSOT가 언급한 필드가 실제 파일에 등장하는지, 파서(`tools/teul-cli/src/cli/gaji.rs::parse_gaji_toml`)가 그 필드들을 실제로 읽는지).
3. `docs/ssot/gaji/`(SSOT 쪽 gaji 스켈레톤)와 실제 `gaji/`(코드베이스)의 구조 차이가 있는지 비교.
4. 가장 단순한 기존 gaji 패키지 1~2개를 "최소 뼈대 예시"로 선정하고 전체 파일 목록+각 파일의 최소 내용을 보고서에 인용.

## 산출물

`docs/context/reports/GAJI_SCAFFOLD_SURVEY_V1.md`:
- 표: `| 패키지명 | 공통 파일 목록 | gaji.toml 필드 실사용 | 비고 |`
- 최소 뼈대 예시 전문 인용 1~2개
- "새 gaji 패키지를 만들려면 최소 몇 개 파일, 어떤 내용이 필요한가" 요약 결론 1문단

## 수용 기준

- [ ] `gaji/` 30개 전수 커버
- [ ] `parse_gaji_toml` 실제 구현이 읽는 필드와 SSOT가 언급하는 필드(det_tier/openness 등)의 일치/불일치 명시
- [ ] 설계 제안 없음(재료 수집만) — `gaji new` 명령의 실제 설계·구현은 Claude가 별도로 진행
- [ ] 코드 수정 없음

## 금지 사항

CLI 명령 구현 / 코드 수정 / 새 gaji 패키지 생성. `codex/queue-20260706` 브랜치에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 산출물: `docs/context/reports/GAJI_SCAFFOLD_SURVEY_V1.md`
- 범위: 실제 `gaji/` 최상위 디렉터리 30개 전수, 재귀 `gaji.toml` 13개, `docs/ssot/gaji/` 재귀 `gaji.toml` 10개 비교.
- 확인한 제품 코드: `tools/teul-cli/src/cli/gaji.rs::collect_packages`, `tools/teul-cli/src/cli/gaji.rs::parse_gaji_toml`.
- 핵심 결과:
  - 현재 제품 스캔은 `gaji/<직계 디렉터리>/gaji.toml`만 집어 최상위 11개만 바로 패키지로 본다.
  - 실제 중첩 `gaji.toml` 2개(`gaji/bogae/space2d`, `gaji/phys/pendulum`)는 현 최상위 스캔 대상이 아니다.
  - 실제 `gaji.toml` 13개는 모두 `id/name/version/ssot_requires/det_tier/openness/description`을 쓰지만, 제품 파서는 `id/name/version`만 읽는다.
  - 관측된 기존 최소 뼈대는 `gaji.toml` + `README.md` 2파일이고, 표준 API 뼈대는 `ddn/exports.ddn`을 더한 3파일 구조다.
- 자기 검증:
  - `gaji/` 30개 전수 표 작성 완료.
  - `parse_gaji_toml` 필드 반영 범위와 SSOT 필드 불일치 명시 완료.
  - 설계 제안, CLI 구현, 코드 수정, 새 gaji 패키지 생성 없음.
