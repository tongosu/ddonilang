# intent

- shared sync merge rule 최소 폐쇄(`priority`, tie-break)를 runtime+pack으로 고정한다.
- tie-break는 `agent_id` 오름차순, 동률 시 입력 슬롯 인덱스 오름차순이다.
- 입력 파일 순서가 바뀌면 provenance(`source_hash`)는 바뀌지만, merge 선택 규칙은 변하지 않아야 한다.
