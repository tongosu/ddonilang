# TASK — teul-cli test: result/proof 산출 + 핀 채움 지원

[목표]
- 교육 팩(L01~)을 VERIFIED로 올리려면, teul-cli가 PASS 시 “proof(해시 증거)”를 기계적으로 제공해야 한다.

[필수 산출]
1) teul-cli test <golden/*.test.json> --emit-result <path>
   - JSON 출력(정본)
   - 최소 필드: name, passed, state_hash, bogae_hash?(opt), trace_hash?(opt), replay_ok, failed_madi?(opt)
2) teul-cli test <...> --print-proof
   - stdout에 파싱 가능한 라인(예: state_hash: blake3:...)

[결정성 규칙]
- repro/diag 생성은 state_hash에 영향 0
- 출력 JSON의 필드 순서/정렬 고정

[선택]
- teul-cli test --pin : PASS 시 test.json에 expected_state_hash “제안(patch)” 출력(직접 수정 금지)
