# gogae8_w78_parallel_realms

W78 병렬 터(Realms) PR-1 기본 팩.

## 케이스
- basic_equal: threads=1/8 동일 결과
- isolation: realm 0만 step, 나머지 realm 해시 불변
- shuffle_input_guard: 입력 배치가 섞여 있어도 정렬 규칙으로 동일 결과

## 실행 예시
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- test pack/gogae8_w78_parallel_realms/input_basic.json --threads 1`
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- test pack/gogae8_w78_parallel_realms/input_basic.json --threads 8`
