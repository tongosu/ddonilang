# 지원 표면 — Language SPEC v0.8.1 공개판

> 후속 교정본: 정본 v25.137.0에 따른 설명 교정. CURRENT·제품 지원·독립 PASS 변경 없음.

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`
> 내부 투영 기준: `v25.137.0 live source / RD-001~690`
> 교정 기준 공개판: `v0.8.0` — 이 파일은 미승격 교정본
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

| 항목 | 문서 의미 | 제품 상태 |
|---|---|---|
| Definition `:=` | v25 공개 정본 | PRODUCT-OPEN |
| Binding/Rebinding `<-` | v25 공개 정본 | PRODUCT-OPEN |
| 새 선언 머리 | v25 공개 정본 | PRODUCT-OPEN |
| typed morphology | 의미·방향 정본 | exact schema·제품 OPEN |
| closed multisurface | RD-480의 7개 구조적 class와 유일 분류 | 물리 연결·제품 지원은 별도 근거 필요 |
| presentation canon | profile 방향 정본 | formatter·registry OPEN |
| pre-v25 migration | migration-only | exact 도구·oracle OPEN |
| state·receipt·lock·wire | 이 판본이 폐기하지 않음 | 별도 감사 OPEN |

`문서 의미`는 제품에서 현재 실행 가능하다는 뜻이 아니다.

| 추가 항목 | 문서 의미 | 제품 상태 |
|---|---|---|
| 수학 다중표면 typed identity | 의미·governance 정본 | exact grammar·제품 OPEN |
| HoTT Path family | Proof 방화벽·의미 계산체계 정본 | 물리 kernel·제품 OPEN |
| HoTT 표면 | RD-508의 3개 표면 확정, 나머지 연구와 분리 | PRODUCT-OPEN / demand-gated |
| 묶음씨 Product definition | successor source 정본 | PRODUCT-OPEN |
| 고름씨 Sum definition·`$갈래` | successor source 정본 | PRODUCT-OPEN |
| typed `[]` Selection | successor source 정본 | exact failures·제품 OPEN |

| Selection 세부 계약 | 문서 의미 | 제품 상태 |
|---|---|---|
| 차림 index/range | 0-based·반열림, 음수·silent default 금지 | exact failure·제품 OPEN |
| 짝맞춤 lookup | strict V 결과, RD-476의 안전 조회 `찾기`는 `(V)있없` | 제품 OPEN |
| 글 selection | grapheme `글` 결과 | normalization·진단 OPEN |
| existing replacement | immutable 교체 + root `<-` | runtime OPEN |
| multi-axis/chained | 서로 다른 typed AST 의미 | Tensor 세부·physical AST OPEN |
| constructor/access/proof | AST·canon category 분리 | parser·formatter OPEN |

| 후대 public projection family | 문서 상태 | 제품 상태 |
|---|---|---|
| Tensor R1 | 의미·기본 selection 표면 | PRODUCT-OPEN |
| ResultPlace·failure/recovery | 의미·successor 표면 | PRODUCT-OPEN |
| PathIR/Proof exact calculus | 의미계약·profile 경계 | kernel/parser/oracle OPEN |
| 상태머신·Regex·말무늬 | RD-448/450/458/475의 한정된 표면 확정 | 고급 확장·물리 구현 OPEN을 항목별 구분 |
| release governance·maturity | 내부 추적 only | public grammar 아님 |

이 표의 제품 OPEN은 이 문서가 실행 지원을 인증하지 않는다는 경계다.
별도 고정 제품의 현재 실행 가능 여부는 그 제품 판본과 실행 근거로 확인한다.
