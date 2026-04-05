# Tests

## Manual run
- `teul-cli canon pack/compound_update_basics/input.ddn`
- `teul-cli run pack/compound_update_basics/input.ddn`
- `teul-cli canon pack/compound_update_basics/input_plus_equal.ddn`
- `teul-cli run pack/compound_update_basics/input_plus_equal.ddn`
- `teul-cli canon pack/compound_update_basics/input_minus_equal.ddn`
- `teul-cli run pack/compound_update_basics/input_minus_equal.ddn`
- `python tests/run_pack_golden.py compound_update_basics`

## Expected
- canon 출력에서 `바탕.점수 <- 바탕.점수 + 5.` 및 `바탕.점수 <- 바탕.점수 - 3.` 형태로 전개된다.
- run 출력은 `12`이다.
- `+=`/`-=` 입력은 canon에서 `E_CANON_UNSUPPORTED_COMPOUND_UPDATE`, run에서 `E_PARSE_UNEXPECTED_TOKEN`으로 거부된다.
