# D-PACK: open_file_read_path_normalize

## 목적
- file_read 경로의 `./` 정규화가 replay 매칭에 반영되는지 확인한다.

## 구성
- `input.ddn`: `./` 경로로 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: 정규화된 key 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드

## WARNING (public snapshot)
- 이 경로는 정본 pack authoring 경로가 아니다.
- 정본은 `pack/open_file_read_path_normalize`를 우선 참조하고, 필요 시 `docs/ssot/pack/open_file_read_path_normalize`를 함께 참조한다.
- 정책/허용목록/차단 규칙 또는 경로 정규화 규칙은 runtime/live reference와 함께 읽어야 한다.
- 이 스냅샷만 보고 운영/보안/파일경로 정책을 확정하면 안 된다.
- 새 작업의 시작점으로 사용하지 않는다.
