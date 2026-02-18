# 테스트

## 실행
- `cargo run -p teul-cli -- run pack/gogae4_w43_audit_inspector/input.ddn --madi 6 --seed 0x0 --sam pack/gogae4_w43_audit_inspector/sam/left_pulse.input.bin --geoul-out build/geoul/w43 --trace-tier T-OFF`
- `cargo run -p teul-cli -- geoul query --geoul build/geoul/w43 --madi 3 --key 살림.점수`
- `cargo run -p teul-cli -- geoul backtrace --geoul build/geoul/w43 --key 살림.점수 --from 0 --to 5`
