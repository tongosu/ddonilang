> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 또니랑 Language SPEC v0.9.0 — 현행 결론 뷰

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

이 후속 family는 v25.151.0의 승격 의미를 직접 설명한다. 이전 공개판을 먼저 읽거나 RD691~709 부록에서 우선권을 조립할 필요 없이 아래 주제별 본문을 읽는다. 자세한 결정 인용은 추적표에 보존한다. 후보·미승격 논의는 후속안 문서에 분리한다.

## 1 선언·호출·타입

Definition :=와 값 결속/다시결속은 다르다. 완전 선언은 정적 문맥·필수 pin·지역 binder·타입·결과·종류·본문을 하나의 정의 정체에 결속한다. 호출마다 원문 순서로 인수를 평가하고 fresh callee pin을 연결한다. 등록 표면의 같은 철자가 다른 역할/소유를 자동 통합하지 않는다.

C3는 정확 등록 집합이고 C2는 선택적·명시·유한·판본 고정 전개다. morphology·순서·유사도·AI 추측으로 의미를 공급하지 않는다. source 보존과 명시 canon, semantic identity를 구분한다. Product/Sum/Selection/Tensor의 각 owner가 타입·선택·실패·갱신을 소유한다.

상세: [문법](SPEC_LANGUAGE_SYNTAX_SUMMARY_v0.9.0.md), [의미 핵심](SPEC_SEMANTIC_CORE_v0.9.0.md), [한국어·현지화 권위](SPEC_KOREAN_MASTER_AND_LOCALIZATION_AUTHORITY_v0.9.0.md), [용어](SPEC_GLOSSARY_v0.9.0.md).

## 2 결과·실패·저장

이름 있는 결과 callable은 invocation-local ResultPlace를 정상 종료의 모든 경로에서 채운다. 기록이 즉시 return은 아니다. typed 안됨은 hard failure와 다르며 원래 맺음 전체 전달과 마침은 현재 callable에만 작용한다. 자동 wrapping·숨은 propagation·복구 RHS의 미확정 확대는 없다.

입력 조회는 admitted snapshot의 비소비 관찰이다. 저장 트리는 host stack을 의미 한도로 삼지 않고 기존 byte/count 계약을 지킨다. typed text/bool 경로와 legacy clean-break, pre-CAS 보존과 post-CAS/불확실 RecoveryRequired를 구별한다. fresh durable 확인과 전체 custody 재검증 전 성공을 앞당기지 않는다.

상세: [결과·실패](SPEC_CALLABLE_RESULT_FAILURE_R1_v0.9.0.md), [호환성](SPEC_COMPATIBILITY_AND_NONCLAIMS_v0.9.0.md), [이관](SPEC_MIGRATION_v0.9.0.md).

## 3 수학 함수·보임

조건별 함수는 선언 D 전체를 덮고 겹침은 값이 같아도 거부하며 D 밖 조각은 자동 절단하지 않는다. 명시 빈 D와0행만 빈 함수로 표시한다. 공통 binder/carrier·정의 가능성·제한 D·정확 증거를 보존한다.

보임의 직접 괄호쉼표 생성은 유일하게 해소된 이름 있는 결과 목표 범위다. 모든 항을 검사한 뒤 투영/후속 소비한다. ordinary Tuple이나 성공 플래그가 보임이 아니고 일반 runtime/저장 Proof 지원을 새로 열지 않는다.

상세: [조건별](SPEC_PIECEWISE_FUNCTION_v0.9.0.md), [보임](SPEC_PROOF_KERNEL_PROFILES_v0.9.0.md), [연구 표면](SPEC_HOTT_RESEARCH_SURFACE_v0.9.0.md).

## 4 실행·재열기·재생·비교

시작은 새 의미 실행의 첫 실제 마디에만, 매마디는 첫 마디 포함 각 마디에 적용한다. 같은 진입 단위의 root/hook은 원문 순서다. 시작 hoist/phase/dedup은 없다. 저장 재열기·continue·새 record epoch는 시작 재발동 권한이 아니다. 격리 원 seed 재생은 원 계산을 재현하지만 원세계 effect를 추가 공개하지 않는다.

정확 비교는 관찰·대상/사건 대응·논리 시점·구간 끝점·필수 기록 완전성에 결속한다. 누락/잘림/원 실패를 성공으로 메우지 않는다. 근사 비교는 후속이다.

상세: [lifecycle·비교](SPEC_LIFECYCLE_AND_EXACT_COMPARISON_v0.9.0.md), [결정성](SPEC_DETERMINISM_CONTRACT_v0.9.0.md).

## 5 대체·지원·비현행

부분 대체된 범위만 후속 규칙을 적용하며 나머지 의미를 자동 삭제하지 않는다. RD001~053의 현행 개별 계승과2025년12월 전량 보존은 UNKNOWN이다. 규격 의미와 실제 제품 지원·환경별 실행·독립검토·Human·공개를 각각 표시한다. 공식26/28·P5/P6 OPEN은 언어 지원 수치가 아니다.

[대체 근거](대체근거와_부분효력.md), [전체 추적표](SPEC_TRACE_v0.9.0.md), [전체 RD 탐색](현행결론_RD색인.md), [제품 근거](SPEC_PRODUCT_EVIDENCE_v0.9.0.md), [비현행 후속안·OPEN](승인된후속안과_OPEN.md), [예제](SPEC_EXAMPLES_v0.9.0.md)를 함께 제공한다.

이 뷰는 RD371/372에 따른 비권위 읽기 projection 후보다. source와 충돌하면 추측으로 빈칸을 메우지 않는다. 일반 schema/제품 뷰 구현 완료나 전역 대체관계 완전성을 주장하지 않는다.

<!-- REVIEWED_R21_BODY_END -->
