# 한국어 master와 localization 권위 — Language SPEC v0.8.1 공개판

> 후속 교정본: 정본 v25.137.0에 따른 설명 교정. CURRENT·제품 지원·독립 PASS 변경 없음.

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`

이 v0.8.1 공개판의 한국어 문서군이 목표 의미 기준 원고다. 비한국어 문서는 이 한국어
master의 projection이며 새 semantic identity나 독립 의미 권위를 만들지 않는다.

문서 번역 수, registered LanguageProfile 수, 제품 지원 언어 수, native Human PASS 수는
서로 다른 분모다. 선행 public-document proposal의 한국어+14개 번역 계획은 문서 제작
범위이며 현행 17-profile strategic research denominator나 제품 지원 선언이 아니다.

번역은 preferred surface와 자연스러운 설명을 현지화할 수 있지만 다음을 바꾸지 못한다.

- DefinitionIdentity, ParamPinIdentity, type/result/effect contract
- fail-closed 조건과 deterministic selection
- exact source surface를 닫은 owner
- OPEN, DEFERRED, product/release nonclaim

번역에서 의미 충돌이나 exact surface 불확실성이 발견되면 한국어 master를 추측 수정하지
않고 별도 correction proposal로 반환한다. 번역 완료는 v0.8.1 공개판의
독립검토를 대신하지 않으며 Human PASS를 자동 부여하지 않는다.

## 명시 등록 표면과 morphology 비권위

RD-570에 따라 `A{A1~...~An}`은 explicit registered accepted-surface 집합이다.
자연어 문법성·규칙성·파생 관계와 morphology profile은 admission 권위가 아니다.
profile은 작성 편의를 제공할 수 있지만 semantic identity, veto, hidden generation,
fallback을 소유하지 않는다. 이 제한은 아래 LanguageProfile·DDL 설명에도 적용된다.

## DDL은 구현 번역이 아닌 공개 선언 투영

RD-571/574~577에 따라 DDL은 동일한 DefinitionIdentity의 언어별 공개 선언이다.
구현 본문을 복제하거나 새로운 의미 owner를 만들지 않는다. type parameter·요구조건,
pin identity와 의미 순서, pin type/default, 결과·effect·kind를 보존한다.
언어별 글의 순서와 의미 pin 순서는 별개다.

`.ddn`은 파일당 단일 언어이고 `.ddl`·`.옮김`은 서로 다른 정확한 profile의
여러 section을 가질 수 있다. 동일 profile 중복·미등록 profile은 실패한다.
RD-628의 profile당 canonical header 하나와 명시 alias 집합 규칙을 따른다.
현재 모든 언어의 exact header 철자·registry/schema/wire가 확정됐다는 뜻은 아니다.

정확히 ASCII 점 3개인 `:={...}`는 해석 완료된 DDL payload 자리에서만 callable
본문 또는 immutable value initializer를 생략한다. public type·field·branch·default
schema를 감추는 수단이 아니며 ordinary `.ddn`에는 생략 의미가 없다.

DDL의 완전성은 지원을 선언한 언어별 PublicGajiSurface의 identity 집합을 기준으로
검사한다. 지원하지 않는 언어를 자동 fallback하지 않는다. 17개 연구 profile 전체의
제품 지원을 한꺼번에 주장하지 않는다. 정확한 DDL grammar·AST·검증 입력과
parser/canon/IDE 연결은 미완료 경계로 남긴다. 일반적인 번역 도구가 이를 대신하지 않는다.

## 선언작성 도구·검증·갱신 경계

언어 설계의 명세 보존, 설계 상태와 제품 지원 상태는 서로 다르다. 미구현 의미를
문서에서 지우지 않으며 고위험 부분은 작은 실험으로 확인한다. 모든 언어 문서를
완성할 때까지 제품 진행을 대기시키지 않는다.

선언작성 도구는 기존 원본 선언 골격을 추출할 수 있고 사람 또는 AI는 언어별 표면
초안을 작성할 수 있다. 결정적 검증기는 기존 identity·type·default·result·effect·kind와
public schema를 대조한다. AI와 초안 도구는 의미 발급·확정 권위가 아니며 이 절은
새 public syntax나 registry를 승인하지 않는다.

DDL은 exact 원본 결속을 유지하고 명시적 갱신 때 공개 계약 차이를 검사해 재사용한다.
내부 구현 변경만으로 전부 재번역하지 않지만 pin·type·default·effect 변경은 영향 검사를
수행한다. automatic latest/fallback은 금지하며 기존 revision/identity owner를 재사용한다.
새 hash 또는 호환 정책을 이 문서에서 만들지 않는다.

현재 제품 입력으로 남은 것은 exact profile header registry, 파일·section·선언·문자열·주석
경계와 escaping을 포함한 전체 grammar, physical AST와 identity 결속, 오래된 DDL의 판정,
성공·실패·canon 실행 corpus다. RD-537의 `쓰임`-`옮김` 활성화와 artifact 골격 및
RD-571~577/628의 의미 owner를 재사용한다. 이것을 효과 있는 일반 프로그램 인자의
원문 평가 순서를 바꾸는 자동번역 권위로 해석하지 않는다.
