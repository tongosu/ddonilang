# D-PACK: open_file_read_key_mismatch

## 목적
- replay 모드에서 file_read key(경로) 불일치가 `E_OPEN_REPLAY_MISSING`으로 진단되는지 확인한다.

## 구성
- `input.ddn`: 파일 읽기 호출 샘플
- `alpha.txt`: 실제 입력 파일
- `open.log.jsonl`: key 불일치 로그
- `golden.jsonl`: 오류 출력 기대
- `tests/README.md`: 수동 실행 가이드

## WARNING (public snapshot)
- 이 경로는 정본 pack authoring 경로가 아니다.
- 정본은 `pack/open_file_read_key_mismatch`를 우선 참조하고, 필요 시 `docs/ssot/pack/open_file_read_key_mismatch`를 함께 참조한다.
- 정책/허용목록/차단 규칙/스키마 규칙은 runtime/live reference와 함께 읽어야 한다.
- 이 스냅샷만 보고 운영/보안/경로 정책을 확정하면 안 된다.
- 새 작업의 시작점으로 사용하지 않는다.
