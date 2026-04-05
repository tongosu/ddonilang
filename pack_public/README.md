# pack_public

## 경고 (운영 라벨)

이 경로는 정본 D-PACK 축이 아니다.

- 성격: legacy/support 성격의 공개 스냅샷 보조 경로
- 금지: 새 작업의 시작점으로 사용하거나 closure 근거로 사용
- 기준 정본: `pack/` (내부 D-PACK 축), `docs/ssot/pack/` (SSOT 관리 축)

Round 1.5 정책:
- `pack_public`을 새 축으로 승격하지 않는다.
- 실경로 대이동/삭제 없이 안내/라벨 정리만 수행한다.

이 폴더는 공개용 pack 스냅샷이다.
- 비정본 입력(alt/alias/legacy/error/fail/replay/record 등)은 제외한다.
- UNC 경로(//SERVER 등) 포함 입력은 제외한다.
- 포함된 input.ddn은 teul-cli canon으로 정본화된 결과다.

포함/제외 통계는 README 하단에 기록한다.

## 통계
- 포함: 126
- 제외: 12
- 실패: 24
