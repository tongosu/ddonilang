# seamgrim_wasm_cli_runtime_parity_v1

`teul-cli`를 oracle로 두고 WASM/셈그림 제품 경로가 같은 current-line 실행 결과를 내는지 확인하는 parity pack이다.

`state_hash`는 1차에서 report-only다. 사용자 체감 결과인 stdout, 관찰 row, resource value JSON, diagnostics를 우선 strict 비교한다.
