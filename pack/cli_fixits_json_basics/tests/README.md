# Tests

## Manual run
- `teul-cli canon --emit fixits-json pack/cli_fixits_json_basics/input.ddn`

## Expected
- JSON array with four fix-it suggestions (변수/함수/클래스/이벤트).
- Deterministic ordering by file/line/col.
