# tools/ledger_seed

목적:
- SSOT 문서들에서 커버리지 장부 씨앗(autoseed)을 생성합니다.
- 생성 결과는 docs/guides/ddonirang_mastercourse/_meta 아래에 저장됩니다.

사용:
- python tools/ledger_seed/seed_from_ssot.py --version 20.2.9 --out docs/guides/ddonirang_mastercourse/_meta/COVERAGE_LEDGER_AUTOSEED_v20.2.9.md

원칙:
- 출력은 결정적이어야 합니다(동일 입력 → 동일 파일).
- 생성 파일은 “ORPHAN 기본”으로 두고, 사람이 귀속/증거를 채웁니다.
