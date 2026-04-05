# D-PACK: open_deny_policy

## 목적
- `--open=deny`에서 열림 호출이 차단되는지 확인한다.

## 구성
- `input.ddn`: 열림 호출 샘플
- `golden.jsonl`: deny 오류 출력 기대
- `tests/README.md`: 수동 실행 가이드

## WARNING (public snapshot)
- 이 경로는 정본 pack authoring 경로가 아니다.
- 정본은 `pack/open_deny_policy`를 우선 참조하고, 필요 시 `docs/ssot/pack/open_deny_policy`를 함께 참조한다.
- 정책/허용목록/차단 규칙은 runtime/live reference와 함께 읽어야 한다.
- 이 스냅샷만 보고 운영/보안 정책을 확정하면 안 된다.
- 새 작업의 시작점으로 사용하지 않는다.
