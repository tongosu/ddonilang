# D-PACK: project_age_target_error_code_example

목적: `expected_error_code`를 사용하는 golden.jsonl 예시를 제공한다.

## 케이스
- K001_invalid_project_value_EXPECT_FAIL
  - project age_target invalid → E_PROJECT_AGE_TARGET_INVALID
- K002_invalid_cli_value_EXPECT_FAIL
  - CLI age_target invalid → E_CLI_AGE_TARGET

## 실행
python tests/run_pack_golden.py project_age_target_error_code_example