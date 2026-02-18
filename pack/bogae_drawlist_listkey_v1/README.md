# D-PACK: bogae_drawlist_listkey_v1

## 목적
- 보개 목록 키 `보개_그림판_목록`의 렌더 경로를 검증한다.
- 목록 항목(묶음)에서 Rect/Text/Sprite를 수집해 결정적으로 렌더해야 한다.

## 구성
- `input.ddn`: 목록 키로 사각형 1개를 렌더
- `tests/README.md`: 수동 실행 가이드

## DoD(최소)
- `bogae_hash`가 결정적으로 재현된다.
