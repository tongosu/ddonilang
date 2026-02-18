# D-PACK (NEGATIVE): project/project_age_target_precedence_v1_negative

목적: age_target 결정 우선순위(pack)에서 FAIL expected 케이스를 분리한다.

## 케이스
- K004_invalid_project_value_EXPECT_FAIL: project age_target invalid → 결정적 오류 (E_PROJECT_AGE_TARGET_INVALID)
- K005_invalid_cli_value_EXPECT_FAIL: CLI age_target invalid → 결정적 오류 (E_CLI_AGE_TARGET)

## 검증 규칙
- 실패가 정답.
- 각 케이스는 `golden.jsonl`에서 `expected_error_code`를 반드시 명시한다.
- runner 판정:
  - `exit_code`가 0이 아니어야 한다.
  - stderr(또는 stdout)에서 `expected_error_code`가 포함되어야 한다.
