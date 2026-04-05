# showcase_tetris_mini_v1

현재 Gate0 실행기에서 바로 동작하는 테트리스 미니 쇼케이스 팩.

- 목적: 진자 쇼케이스와 함께 "게임형 보개" 결과물을 즉시 실행/확인.
- 범위: 풀 게임 로직(충돌/라인클리어) 대신 입력 가능한 테트로미노 렌더/낙하 애니메이션.

## 입력 키
- `left`: 좌 이동
- `right`: 우 이동
- `down`: 가속 낙하
- `up`: 회전(90도)
- `enter`: 하드 드롭

## 실행 예시
- 웹:
  - `teul-cli run pack/showcase_tetris_mini_v1/input.ddn --seed 0x0 --madi 240 --bogae web --bogae-live --sam-live web --bogae-out out/showcase/tetris_mini_web --no-open`
- 콘솔:
  - `teul-cli run pack/showcase_tetris_mini_v1/input.ddn --seed 0x0 --madi 240 --bogae console --bogae-live --sam-live console --console-grid 23x25 --console-cell-aspect 2:1 --console-panel-cols 0 --no-open`
