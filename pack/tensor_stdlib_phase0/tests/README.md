# Tests

## Manual run
- `teul-cli run pack/tensor_stdlib_phase0/input.ddn`

## Expected
- 출력 순서:
  1) `차림[2, 2]`
  2) `차림[1, 2, 3, 4]`
  3) `가로먼저`
  4) `2`
  5) `없음`
  6) `묶음{배치=가로먼저, 자료=차림[1, 2, 9, 4], 형상=차림[2, 2]}`
