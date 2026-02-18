# D-PACK: gogae3_w27_invariant_hook

## 목적
- 제3고개(23~33) 관련 팩 스켈레톤입니다.
- 완료 판정은 **팩 통과(state_hash / bogae_hash)**로 합니다.

## 구성(권장)
- input.ddn : 세계/시나리오(또는 엔트리) 정의
- inputs/    : 입력샘(detjson/bin 등)
- expect/    : 기대 해시/기대 출력
- diag/      : 해시에 영향 없는 진단(로그/요약)
- tests/     : 실행 커맨드/골든 비교

## DoD(최소)
- 동일 입력샘 리플레이 -> 동일 state_hash
- 동일 DrawList 생성  -> 동일 bogae_hash
