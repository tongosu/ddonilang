# Tests

## Manual run
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- run pack/seamgrim_line_graph/input.ddn --bogae-out C:/ddn/codex/build/seamgrim_line_graph.bdl1`
- `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- run pack/seamgrim_line_graph/input_parabola.ddn --bogae-out C:/ddn/codex/build/seamgrim_line_graph_parabola.bdl1`

## Expected
- `bogae_hash`가 runner 출력으로 검증된다.