# D-PACK: open_decl_policy_warn

## 목적
- `#열림 허용(...)`에 없는 open_kind 호출 시 경고가 나오는지 확인한다.

## 구성
- `input.ddn`: 시각만 허용하고 파일읽기를 호출
- `hello.txt`: 파일 읽기 입력
- `open.log.jsonl`: replay용 로그(결정적 고정값)
- `tests/README.md`: 수동 실행 가이드

## WARNING (public snapshot)
- 이 경로는 정본 pack authoring 경로가 아니다.
- 정본은 `pack/open_decl_policy_warn`를 우선 참조하고, 필요 시 `docs/ssot/pack/open_decl_policy_warn`를 함께 참조한다.
- 정책/허용목록/차단 규칙/스키마 규칙은 runtime/live reference와 함께 읽어야 한다.
- 이 스냅샷만 보고 운영/보안/경로 정책을 확정하면 안 된다.
- 새 작업의 시작점으로 사용하지 않는다.
