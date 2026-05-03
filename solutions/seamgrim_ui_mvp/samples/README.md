# seamgrim_ui_mvp samples

이 폴더는 셈그림 current-line exemplar sample rail을 담는다.

## canonical first-run path

- 처음 시작은 hello -> 움직임 -> slider -> replay/거울 순서로 본다.
- 이 README와 `onboarding.json`은 같은 canonical first-run path를 가리킨다.

1. hello
   - `06_console_grid_scalar_show.ddn`
   - 가장 짧은 one-shot console-grid 시작점
2. 움직임
   - `09_moyang_pendulum_working.ddn`
   - `모양 {}` + `space2d` representative path
3. slider
   - `onboarding.json`
   - `매김` 기반 조절 path
4. replay/거울
   - `onboarding.json`
   - 마지막 step에서 replay/거울 읽기 rail

## 실험 샘플

- `15_console_grid_maze_probe.ddn`
  - `보개종류 <- "콘솔격자"`로 text-grid 콘솔 보개를 명시하고, P/G 미로 상태를 마디별로 확인한다.
- `16_space2d_bounce_probe.ddn`
  - `모양 {}`으로 벽과 공을 그리고, `보임 {}` 그래프 rows로 x/y 위치 변화를 함께 확인한다. 화면 표시 이름은 `평면 공 튕김 실험`이다.
  - 벽 충돌은 공 반지름을 포함한 좌표 기준으로 맞춘다. space2d 보개의 x축 눈금, zoom/pan 표시, 벽 선 위치가 그래프 rows와 어긋나지 않는지 확인하는 회귀 샘플이다.
- `grid2d_v0/maze_probe.grid.detjson`
  - 작업실 DDN 샘플이 아니라 네이티브 grid2d 타일/actor 렌더러 fixture다. 현재 작업실 주보개가 grid2d DDN을 직접 받는다고 주장하지 않는다.

## notes

- `index.json`은 sample inventory 스키마를 유지한다.
- `06_console_grid_scalar_show`, `09_moyang_pendulum_working`에는 `first_run_path` / `tags`가 붙어 canonical first-run entry를 함께 가리킨다.
- canonical first-run path는 이 README와 `onboarding.json`을 함께 읽는 기준으로 둔다.
- `01_*`, `04_*` 계열은 과거 호환 샘플로 남아 있으며, 작업실 예제 탭의 현재 rail은 `index.json`에 등록된 06번 이후 샘플을 기준으로 둔다.
- full polished onboarding UI를 주장하지 않는다.
