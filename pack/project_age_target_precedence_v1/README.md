# project_age_target_precedence_v1

age_target 우선순위를 검증하는 회귀 팩이다.

## 목표
- CLI > ddn.project.json > default 우선순위 확인
- age_target 결정 값/출처가 run_manifest에 기록됨을 확인

## 케이스
- K001_default_only: ddn.project.json에 age_target 없음 → default
- K002_project_over_default: project age_target 적용
- K003_cli_over_project: CLI age_target가 project보다 우선

## 실행
python tests/run_pack_golden.py project_age_target_precedence_v1
