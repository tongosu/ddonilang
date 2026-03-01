# pack/edu_simfirst_lint_v1

SimFirst 교과 계약을 **정적 린트**로 강제하기 위한 D-PACK 성격의 도구입니다.

## 실행

```bash
python pack/edu_simfirst_lint_v1/tools/edu_simfirst_lint.py --root docs/ssot/pack --mode error
```

## 동작

- `docs/ssot/pack/edu_*` 디렉터리를 스캔합니다.
- `lesson.ddn`에 `#교과모드: simfirst.v1` 마커가 있는 교과만 강제 대상입니다.
- 규칙:
  - time/tick 마커(시작/매마디/마디번호/델타시간/게임시간) 최소 1개
  - view 마커(보개_그림판_목록 또는 그래프 접두 키) 최소 1개
  - README에 `## 셈그림 연출` 섹션

## 확장 아이디어

- Gate0 parser가 안정되면 AST 기반으로 “진짜 마디-상태 갱신” 여부를 검사하도록 강화합니다.
