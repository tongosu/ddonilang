# Tests

서브픽셀/AA 정책은 W22 detbin 골든으로 검증한다.

## Manual run
- `python tools/teul-cli/tests/run_golden.py --root tools/teul-cli/tests/golden --walk 22`

## Covered
- AA=0/1 플래그 허용 및 결정성
- Q24.8 소수부 좌표/두께 허용
