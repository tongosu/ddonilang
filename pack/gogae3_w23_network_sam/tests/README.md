# tests

W23 골든 테스트 계획(구현 후 추가).

## 케이스(초안)
- 동일 입력샘 리플레이 -> 동일 state_hash
- sender/seq 섞임 입력을 정렬 후 동일 state_hash
- sample_input_snapshot_unsorted.detjson 파싱 결과가 (sender, seq) 오름차순으로 정렬됨

## 실행(예시)
- cargo test -p ddonirang-tool w23_network_sam_state_hash_is_stable
- teul-cli run --sam <input_snapshot.detjson>
- teul-cli replay --sam <input_snapshot.detjson>
