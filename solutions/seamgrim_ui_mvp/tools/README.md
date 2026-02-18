# tools

## export_graph.py
DDN 출력(숫자 x/y 라인) → `seamgrim.graph.v0` JSON으로 변환합니다.

예시:
- `python solutions/seamgrim_ui_mvp/tools/export_graph.py solutions/seamgrim_ui_mvp/samples/01_line_graph_export.ddn C:/ddn/codex/build/seamgrim_graph.json`
- `python solutions/seamgrim_ui_mvp/tools/export_graph.py solutions/seamgrim_ui_mvp/samples/02_parabola_export.ddn --output-dir C:/ddn/codex/build --auto-name`
- `python solutions/seamgrim_ui_mvp/tools/export_graph.py solutions/seamgrim_ui_mvp/samples/01_line_graph_export.ddn --output-dir C:/ddn/codex/build --auto-name --label-from-input`

출력된 JSON은 `solutions/seamgrim_ui_mvp/ui/index.html`에서 불러올 수 있습니다.
출력 라인은 `x`, `y`를 한 줄에 `x,y`로 출력하거나, 줄마다 숫자 1개씩 출력해도 됩니다.
`#이름:`/`#설명:` 메타 헤더는 실행에 영향을 주지 않으며, 해시 계산에서는 무시됩니다.
그래프 출력은 키 정렬/소수 4자리 반올림으로 결정성을 유지합니다.

## bridge_server.py
UI에서 입력한 DDN을 **로컬에서 실행**하고 결과 그래프를 반환하는 브리지 서버입니다.

실행:
- `python solutions/seamgrim_ui_mvp/tools/bridge_server.py`
- 브라우저에서 `http://localhost:8787/` 접속
 - (옵션) `TEUL_CLI_WORKER=1` 설정 시 `teul-cli worker`(DetJson-RPC) 경로를 사용

UI에서 “DDN 실행(브리지)” 버튼을 누르면 `/api/run`으로 전송되어 결과 그래프가 로드됩니다.
수식/범위 패널의 “DDN 실행”은 수식/범위를 DDN으로 자동 생성해 브리지로 실행합니다.
“로컬 미리보기”는 브리지 없이 JS로 계산한 결과를 그립니다.

## bridge_check.py
브리지 서버 기동/응답을 자동 점검합니다.

실행:
- `python solutions/seamgrim_ui_mvp/tools/bridge_check.py`

내부적으로 서버를 필요 시 기동하고 샘플 DDN을 `/api/run`에 요청합니다.

## export_space2d.py

DDN 출력에서 `seamgrim.space2d.v0` JSON을 추출합니다. DDN stdout의 `space2d`/`공간` 마커와 shape 블록을 파싱합니다.

예시:

- `python solutions/seamgrim_ui_mvp/tools/export_space2d.py input.ddn output_space2d.json`
- `python solutions/seamgrim_ui_mvp/tools/export_space2d.py input.ddn --output-dir C:/ddn/codex/build --auto-name`

## export_text.py

DDN 출력에서 `seamgrim.text.v0` JSON을 추출합니다. DDN stdout의 `text`/`문서`/`해설` 마커 블록을 파싱합니다.

예시:

- `python solutions/seamgrim_ui_mvp/tools/export_text.py input.ddn output_text.json`
- `python solutions/seamgrim_ui_mvp/tools/export_text.py input.ddn --output-dir C:/ddn/codex/build --auto-name`

## export_table.py

DDN 출력에서 `seamgrim.table.v0` JSON을 추출합니다. DDN stdout의 `table`/`표` 마커 뒤 CSV/TSV 형식 데이터를 파싱합니다.

예시:

- `python solutions/seamgrim_ui_mvp/tools/export_table.py input.ddn output_table.json`
- `python solutions/seamgrim_ui_mvp/tools/export_table.py input.ddn --output-dir C:/ddn/codex/build --auto-name`

## lesson_pack_check.py
교과 lesson pack의 `view_spec.toml`에 정의된 `required_views/required_gauges`를 검증합니다.

실행:
- `python solutions/seamgrim_ui_mvp/tools/lesson_pack_check.py`
- 개별 pack 지정: `python solutions/seamgrim_ui_mvp/tools/lesson_pack_check.py pack/edu_s1_function_graph`
