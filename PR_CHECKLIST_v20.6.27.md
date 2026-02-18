# PR 체크리스트 — SSOT v20.6.27 스냅샷 반영

## 0) 전제
- 이번 패치 ZIP은 **추가/변경 파일만 포함**합니다.
- 이전 스냅샷(`v20.6.26`) 파일의 **삭제/이동(git mv)** 는 아래 체크리스트대로 PR에서 수행하세요.

## 1) 버전 스냅샷 이동(권장: releases 보관)
> 아래는 예시 경로입니다. 프로젝트 정책에 맞게 조정 가능.

```bash
mkdir -p docs/ssot/releases/v20.6.26
git mv docs/ssot/ssot/*_v20.6.26.md docs/ssot/releases/v20.6.26/
```

## 2) 새 스냅샷 추가/갱신
- ZIP을 repo 루트에 풀면 아래 파일들이 추가됩니다.
  - `docs/ssot/ssot/*_v20.6.27.md`
  - `docs/guides/SEAMGRIM_EXAMPLE_RULES.md`

## 3) 확인 포인트
- `docs/ssot/ssot/SSOT_ALL_MANIFEST_v20.6.27.md` 의 sha256 검증이 통과하는지 확인
- `SSOT_TOOLCHAIN_v20.6.27.md` 의 `§TC0.3.1a.1` (Seamgrim WASM Bridge API v1) 스키마가 최신인지 확인
- `SSOT_LANG_v20.6.27.md` 의 10.6 절(말씨: 부품 vs 문장) 규칙이 문서/예제에 반영되는지 확인
