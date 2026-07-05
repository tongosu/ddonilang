# BRIEF: DEV_SUMMARY.md 아카이브 분리

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 성격: 순수 정리(hygiene). 판단 없음.

## 배경

Q27에서 `docs/context/all/DEV_SUMMARY.md`가 처음 git에 올라갔는데(기존에 미추적 상태로 25,472줄이 쌓여 있었음), 파일 목적("최신 의사결정/검증 결과를 빠르게 확인하는 용도")에 비해 지금 너무 크다.

## 작업

1. `docs/context/all/DEV_SUMMARY.md`를 읽고, 최근 30일(또는 최근 항목 기준 적절한 개수 — 최소 최근 20개 `###` 항목) 이내 항목만 남기고, 나머지는 `docs/context/all/DEV_SUMMARY_ARCHIVE_20260706.md`로 이동한다.
2. 이동 시 항목 순서/내용은 절대 변경하지 않는다 — 잘라서 붙이는 것뿐이다.
3. `DEV_SUMMARY.md` 최상단에 `> 이전 기록은 DEV_SUMMARY_ARCHIVE_20260706.md 참고` 한 줄만 추가한다.
4. 다른 파일이 `DEV_SUMMARY.md`의 특정 옛 항목을 참조하는지 확인한다(`rg` 검색) — 참조가 있으면 그 파일도 새 아카이브 경로를 가리키도록 갱신할지 보고만 하고 수정은 하지 마라(범위 판단은 Claude).

## 검증

- 분리 전/후 두 파일의 총 줄 수 합이 원본과 일치(내용 손실 없음 확인)
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS

## 수용 기준

- [ ] `DEV_SUMMARY.md`가 최근 항목만 남고 크게 줄어듦
- [ ] `DEV_SUMMARY_ARCHIVE_20260706.md`에 나머지 전부 보존, 내용 손실 없음
- [ ] 참조 파일 있으면 목록만 보고(수정 안 함)
- [ ] sanity gate PASS

## 금지 사항

내용 삭제/수정 없음(이동만). main 직접 커밋 금지.

## 보고 형식

이 파일 하단 `## 실행 보고`: 분리 전/후 줄 수, 남긴 항목 개수, 참조 파일 목록(있으면).
