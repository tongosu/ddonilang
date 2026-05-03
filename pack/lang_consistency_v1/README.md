# lang_consistency_v1

- 상태: ACTIVE
- 주제: 언어 표기/토큰 일관성 회귀
- 범위:
  - `입력키` 계열(`입력키`, `입력키?`, `입력키!`) 런타임 계약
  - 통신 화살표 문형에서 canon/run 진단 코드 고정 (`E_CANON_EXPECTED_TERMINATOR`, `E_PARSE_UNEXPECTED_TOKEN`)

## 케이스 구성
- `c01_logic_alias_canon`: 기본 입력키/입력 판정 정본화 확인
- `c02_signal_arrow_EXPECT_FAIL`: `a <- 참 ~~> 거짓.` 혼합 문형 실패 코드 고정
- `c03_inputkey_compat_option_run`: compat(`입력키`) + option(`입력키?`) 동작 확인
- `c04_inputkey_strict_missing_EXPECT_FAIL`: strict(`입력키!`) 미정의 실패 코드 고정
- `c05_map_dot_nested_write_canon`: `짝맞춤` 중첩 점대입(`공.속도.x <- ...`) 정본화 확인
- `c06_map_dot_nested_write_run`: 중첩 점대입 실행 결과(쓰기 경로) 확인
- `c07_map_dot_nested_write_missing_key_EXPECT_FAIL`: 중첩 점대입 시 부모 키 누락 에러 코드 고정(`E_MAP_DOT_KEY_MISSING`)
- `c08_map_dot_read_missing_key_EXPECT_FAIL`: 점읽기 시 키 누락 에러 코드 고정(`E_MAP_DOT_KEY_MISSING`)
- `c09_contract_tier_sealed_EXPECT_FAIL`: `AGE2 + det_tier=D-SEALED` 실행 시 미지원 코드 고정(`E_CONTRACT_TIER_UNSUPPORTED`)
- `c10_contract_tier_approx_EXPECT_FAIL`: `AGE2 + det_tier=D-APPROX` 실행 시 미지원 코드 고정(`E_CONTRACT_TIER_UNSUPPORTED`)
- `c11_map_optional_lookup_run`: `찾기?` 안전 조회(`없는 키 -> 없음`) 실행 출력 고정
- `c12_matic_entry_strict_EXPECT_FAIL`: strict 기본모드에서 `매틱:움직씨` 입력 차단 코드 고정(`E_LANG_COMPAT_MATIC_ENTRY_DISABLED`)
- `c13_matic_entry_compat_run`: `--compat-matic-entry` 플래그에서 `매틱:움직씨` 실행 허용 확인
- `c14_receive_hook_outside_imja_EXPECT_FAIL`: `받으면` 훅의 `임자` 본문 한정 위반 코드 고정(`E_CANON_RECEIVE_OUTSIDE_IMJA`, `E_RECEIVE_OUTSIDE_IMJA`)
- `c15_reactive_next_pass_run`: 수신 훅 내부 송신(`~~>`)이 재진입 없이 다음 리액티브 패스에서 처리되는 순서 고정(`123`)
- `c16_receive_hooks_non_consuming_order_run`: 하나의 알림이 4종 훅을 모두 통과(non-consuming)하는 dispatch 순서 고정(`2143`)
- `c17_reactive_no_reentry_fifo_run`: 훅 내부 송신이 현재 dispatch를 재진입하지 않고 FIFO로 후속 처리되는 순서 고정(`12434`)
- `c18_hook_sender_default_current_imja_run`: 훅 내부 sender 생략 송신의 기본 보낸이가 현재 `임자` 이름으로 고정(`관제탑`)
- `c19_hook_send_to_non_imja_EXPECT_FAIL`: 훅 내부 큐 송신이 `임자`가 아닌 수신자를 만날 때 런타임 실패 코드 고정(`E_RUNTIME_TYPE_MISMATCH`)
- `c20_reactive_multi_enqueue_fifo_run`: 훅 내부 다중 송신 enqueue 순서(FIFO: 둘알림→셋알림)가 수신 처리 순서에 그대로 반영되는지 고정(`1234`)
- `c21_reactive_nested_enqueue_bfs_fifo_run`: 중첩 enqueue(둘알림이 넷알림 enqueue)에서도 BFS/FIFO 순서(`12345`)가 유지되는지 고정
- `c22_reactive_same_kind_no_reentry_run`: 같은 알림 종류를 훅 내부에서 enqueue해도 현재 이벤트 처리가 재진입되지 않고 순서(`1323`)가 유지되는지 고정
- `c23_receive_hooks_same_rank_decl_order_run`: 같은 rank에 속한 `받으면` 훅이 선언 순서를 유지한 뒤 다음 rank로 넘어가는지 고정(`12345`)
