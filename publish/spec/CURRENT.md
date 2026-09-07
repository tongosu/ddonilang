# 또니랑 공개 언어 스펙

현재 공개판: [`v0.6.0`](./v0.6.0/SPEC_MANIFEST_v0.6.0.md)

이전 공개판: [`v0.5.0`](./v0.5.0/SPEC_MANIFEST_v0.5.0.md), [`v0.4.0`](./v0.4.0/SPEC_MANIFEST_v0.4.0.md)

최종 승격 입력 보존본: [`v0.6.0-v25.15.0-trace-custody-correction-candidate-r6`](./v0.6.0-v25.15.0-trace-custody-correction-candidate-r6/SPEC_MANIFEST_v0.6.0.md), [`v0.5.0-candidate-r3`](./v0.5.0-candidate-r3/SPEC_MANIFEST_v0.5.0.md), [`v0.4.0-candidate`](./v0.4.0-candidate/SPEC_MANIFEST_v0.4.0.md)

교정·실패 후보 보존본: [`v0.6.0-v25.15.0-full-reprojection-correction-candidate-r5`](./v0.6.0-v25.15.0-full-reprojection-correction-candidate-r5/SPEC_MANIFEST_v0.6.0.md), [`v0.6.0-v25.15.0-full-reprojection-candidate-r4`](./v0.6.0-v25.15.0-full-reprojection-candidate-r4/SPEC_MANIFEST_v0.6.0.md), [`v0.6.0-source-surface-math-hott-product-sum-typed-selection-integration-correction-candidate-r3`](./v0.6.0-source-surface-math-hott-product-sum-typed-selection-integration-correction-candidate-r3/SPEC_MANIFEST_v0.6.0.md), [`v0.6.0-source-surface-math-hott-product-sum-typed-selection-integration-candidate-r2`](./v0.6.0-source-surface-math-hott-product-sum-typed-selection-integration-candidate-r2/SPEC_MANIFEST_v0.6.0.md), [`v0.6.0-source-surface-epoch-candidate`](./v0.6.0-source-surface-epoch-candidate/SPEC_MANIFEST_v0.6.0.md)

- `v0.6.0`은 내부 SSOT `v25.15.0 / RD-001~437`의 공개 언어·governance 범위를
  다시 투영한 docs-first 공개판이다.
- Definition `:=`, Binding/Rebinding `<-`, 새 선언 머리, typed morphology,
  closed multisurface와 presentation canon을 새 공개 source epoch로 삼는다.
- Product/Sum/typed Selection, Tensor R1, callable ResultPlace·failure, PathIR·Proof
  kernel profile의 닫힌 문서 의미를 포함한다.
- R5 독립 검토에서 발견된 trace 계보 불일치는 R6에서 교정됐고, R6는 독립
  `PASS / blocking corrections 0`과 Owner 최종 승격을 거쳤다.
- 내부 SSOT live current는 `v25.15.0`이다. `v0.6.0`은 이 판본의 공개 투영이며,
  이후 내부 successor는 별도 공개 SPEC 승격 전까지 자동 투영하지 않는다.
- `v0.5.0`과 이전 공개판·모든 후보·실패판은 제자리 보존한다.
- pre-v25 source는 일반 parser의 영구 compatibility alias가 아니며 명시적
  migration-only·fail-closed 입력으로 다룬다.
- physical parser·AST·schema·wire·proof kernel·runtime·executable oracle,
  V1 membership·numeric maturity·Human PASS·외부 사용자 성공은 OPEN이다.
- 문서 public current와 제품 구현·runtime landed·기본 배포는 분리한다.
- 내부 SSOT와 공개 Language SPEC은 별도 버전 계보다. 충돌하면 internal current가 우선한다.
