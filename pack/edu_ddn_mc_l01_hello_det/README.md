# edu_ddn_mc_l01_hello_det

목표:
- L01 Hello, Determinism을 팩으로 고정한다.
- “예제”가 아니라 “검증 가능한 단위”로 학습을 끝낸다.

구성:
- lesson.ddn
- inputs/empty.detjson
- golden/L01_hello_det.test.json

실행(예시):
- teul-cli run lesson.ddn ...
- teul-cli test golden/L01_hello_det.test.json

NOTE:
- expected_state_hash 핀은 실행 결과로 채운다.
