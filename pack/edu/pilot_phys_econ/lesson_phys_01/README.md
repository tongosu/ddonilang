# lesson_phys_01 — 진자(바탕/다짐 포함, 길이연산 제거)

포함:
- lesson.ddn: 바탕/다짐(전제/약속) 패턴 포함 + 수식{} + (바인딩)인 ... 풀기
- meta.toml
- expected.scene.v0.json / expected.session.v0.json
- expected.graph.v0.json (생성본 포함) + generate_expected_graph.py(재생성 스크립트)

주의:
- theta 유한성 검사는 `theta == theta`(NaN이면 거짓) 간이 패턴입니다.
- `(그림 길이)` 같은 표준 연산 의존은 제거했습니다.
