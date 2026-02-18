# TASK — COVERAGE_LEDGER auto-seed 생성기 + ORPHAN 리포트 (Codex)

[SSOT 기준]
- Canon SSOT: SSOT_ALL_v20.2.9.md
- SSOT_ALL sha256: 50962b06341e5b4b7a39c108e6a1019abde8ae29ebc17c2ce4a54942c7bbeb69
- 이 작업은 SSOT 본문을 수정하지 않는다. (docs/guides + tools + prompts만)

[목표]
1) SSOT 문서들을 스캔해 “커버리지 장부 씨앗(autoseed)”을 자동 생성한다.
2) 생성된 항목들은 기본적으로 ORPHAN으로 두고,
   사람이 강/부록/코너케이스/impl_notes로 귀속시키며 proof를 채운다.
3) ORPHAN 후보/중복/미확정 정책(TBD)을 리포트로 출력한다.

[산출물]
A) 생성기(스크립트 또는 teul 서브커맨드)
- 권장 경로: tools/ledger_seed/seed_from_ssot.py
- 입력:
  - --ssot-root (기본: repo에서 SSOT_*_v*.md 검색)
  - --version (예: 20.2.9)
  - --max-level (기본 3)
- 출력:
  - docs/guides/ddonirang_mastercourse/_meta/COVERAGE_LEDGER_AUTOSEED_v<ver>.md
  - (선택) build/ledger_seed_report.json

B) 문서
- docs/guides/ddonirang_mastercourse/_meta/LEDGER_SEED_SPEC.md (규격)
- prompts/codex_tasks/TASK_LEDGER_SEED_FROM_SSOT_v20.2.9.md (이 티켓 자체)

[DoD]
- 동일 입력(SSOT 파일들)에서 생성 결과가 bit-level 동일해야 한다(결정적 출력).
- item_id 충돌 0 (중복은 __2, __3로 결정적으로 해소).
- 최소 depth 2~3 헤딩은 100% 포함.
- 출력 Markdown table 정본(필드 순서/정렬) 고정.
- ORPHAN 리포트:
  - ORPHAN 개수
  - class별 분포
  - 섹션 표식(§...) 없는 항목 비율
  - 중복 해소(__n) 항목 목록

[검증]
- tools/ledger_seed/seed_from_ssot.py를 2회 실행해 결과 파일 sha256이 동일함을 확인.
- 다른 OS에서도 동일해야 함(가능하면 CI).

[주의]
- 생성기는 COVERAGE_LEDGER.md를 직접 덮어쓰지 말고,
  AUTOSEED 파일만 갱신하도록 한다. (사람이 귀속/증거를 채우는 주 파일과 충돌 방지)
