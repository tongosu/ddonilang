# Tests

## Manual run
- `teul-cli canon pack/compound_update_basics/input.ddn`
- `teul-cli run pack/compound_update_basics/input.ddn`

## Expected
- canon 출력에서 `살림.점수 <- 살림.점수 + 5.` 및 `살림.점수 <- 살림.점수 - 3.` 형태로 전개된다.
- run 출력은 `12`이다.
