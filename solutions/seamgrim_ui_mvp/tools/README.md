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
legacy `붙박이마련`/`그릇채비`/`채비` 블록은 실행 전 `이름 <- 값.` 대입으로 정규화해 파싱 실패를 줄입니다.

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

## lesson_schema_audit.py
교과 DDN의 레거시/현행 패턴 사용 현황을 스캔합니다.

실행:
- `python solutions/seamgrim_ui_mvp/tools/lesson_schema_audit.py --limit 30`
- json 리포트: `python solutions/seamgrim_ui_mvp/tools/lesson_schema_audit.py --json-out C:/ddn/codex/build/lesson_schema_audit.json`
- preview 포함 스캔: `python solutions/seamgrim_ui_mvp/tools/lesson_schema_audit.py --include-preview`

## lesson_schema_upgrade.py
레거시 `보여주기.` 구문을 `보임 { ... }.` 블록으로 1차 변환하고, 필요 시 `(매마디)마다 { ... }.` 블록을 주입한 preview 파일을 생성합니다.
`*.before_age3_promote.bak.ddn` 백업 파일은 자동 제외합니다.

실행:
- 일부 교과 preview 생성:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --paths math_line high_math_quadratic college_physics_harmonic --include-inputs --write-preview`
- 전체 교과 dry-run:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --include-inputs`
- AGE3 강제 규칙 리포트(매마디/보임):
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --paths math_line --enforce-age3 --json-out C:/ddn/codex/build/lesson_age3_check.json`
- `--write-preview`와 함께 `--enforce-age3`를 쓰면, 변경이 없는 파일도 기존 preview가 있으면 preview 본문 기준으로 검증합니다.
- preview 기준 검증(파일 미수정):
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --include-inputs --inject-mamadi --prefer-existing-preview --enforce-age3 --quiet --summary-out C:/ddn/codex/build/lesson_age3_summary.json`
- 매마디 자동 주입 + preview 생성:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --include-inputs --write-preview --inject-mamadi`
- CI 요약 출력(quiet + summary json):
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --include-inputs --inject-mamadi --enforce-age3 --quiet --summary-out C:/ddn/codex/build/lesson_age3_summary.json`
- UI 필터용 상태 인덱스 생성:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_upgrade.py --include-inputs --inject-mamadi --write-preview --status-out solutions/seamgrim_ui_mvp/lessons/schema_status.json`
  - 생성된 `lessons/schema_status.json`은 UI 교과 탭의 `DDN 상태` 필터에서 사용합니다.

## lesson_schema_promote.py
preview(`*.age3.preview.ddn`)를 source(`lesson.ddn`/`inputs/*.ddn`)로 승격합니다.
`*.before_age3_promote.bak.ddn` 백업 파일은 승격 대상에서 자동 제외합니다.

실행:
- dry-run:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote.py --include-inputs`
- 실제 승격:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote.py --include-inputs --apply --json-out C:/ddn/codex/build/lesson_schema_promote_report.json`
- preview 누락을 실패로 처리:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote.py --include-inputs --fail-on-missing-preview`
- dry-run에서 승격 대기(would_apply) 있으면 실패:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote.py --include-inputs --fail-on-would-apply`
- 대상 목록 파일 사용:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote.py --paths-file C:/ddn/codex/build/promote_targets.txt`

## lesson_schema_promote_flow.py
검증(`upgrade --enforce-age3`) → 승격(promote) → 상태인덱스 갱신을 한 번에 실행합니다.

실행:
- 흐름 dry-run:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote_flow.py --include-inputs`
- 실제 승격 적용:
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote_flow.py --include-inputs --apply`
- 배치 승격(예: pending 50개씩):
  - `python solutions/seamgrim_ui_mvp/tools/lesson_schema_promote_flow.py --include-inputs --apply --batch-size 50 --batch-offset 0`
  - 다음 배치: `--batch-offset 50`, `--batch-offset 100` ...
