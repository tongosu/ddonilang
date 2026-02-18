# pack/game_maker_tetris_full

또니랑(DDN)으로 만든 테트리스 “풀팩” 예제입니다.

## 포함 기능
- 7-bag 랜덤
- 홀드(한 조각당 1회)
- 고스트 조각
- DAS/ARR 이동
- 소프트/하드 드롭 + 드롭 점수
- 락 딜레이
- 라인 점수 + 콤보 + 백투백(B2B)
- 간이 T-Spin 판정/점수
- 레벨/속도 상승

> 키 입력은 `샘.키보드.*`를 사용합니다. 결정적 재생을 위해 `--seed`를 권장합니다.

## 키 매핑
- 왼쪽화살표: 좌 이동
- 오른쪽화살표: 우 이동
- 아래쪽화살표: 소프트 드롭
- 위쪽화살표: 시계 회전
- Z키: 반시계 회전
- X키: 홀드
- 스페이스: 하드 드롭

## 실행 예시
- 웹(데스크톱 브라우저)
  - `teul-cli run pack/game_maker_tetris_full/input.ddn --madi 600 --bogae web --bogae-live --sam-live web --bogae-skin pack/game_maker_tetris_full/skin/tetris_skin_v1.detjson --bogae-out out/tetris_full_live --no-open`
  - `(cd out/tetris_full_live) python -m http.server 8000`
  - `http://localhost:8000/viewer/live.html?input=http://127.0.0.1:5001/input&scale=1.25`

- CLI(콘솔 격자)
  - `teul-cli run pack/game_maker_tetris_full/input.ddn --madi 600 --bogae console --console-grid 23x25 --console-cell-aspect 2:1 --console-panel-cols 0 --sam pack/game_maker_tetris_full/sam/tetris_full.input.bin --no-open`

- 시드 고정
  - `teul-cli run pack/game_maker_tetris_full/input.ddn --seed 0x0 --madi 120 --bogae web --bogae-out out/tetris_full_seed --no-open`

## 참고
- `skin/`에 테트리스 타일 스킨이 포함되어 있습니다.
- `sam/tetris_full.input.bin`은 샘 입력 예시입니다.
- 콘솔/웹 정렬 규칙은 `docs/guides/TETRIS_RENDER_ALIGNMENT.md` 참고.
