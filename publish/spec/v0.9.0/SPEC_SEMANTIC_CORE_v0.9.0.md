> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 의미 핵심 — Language SPEC v0.9.0 후속 후보

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

## Definition identity

Definition은 선언 머리와 본문을 결속한 이름 있는 의미 객체다. alternate lexical
form이나 presentation surface는 별도 Definition을 만들지 않는다.

## Binding/Rebinding

ordinary binding과 rebinding은 값의 현재 결속을 소유한다. persistent update는
새 값을 만든 뒤 최종 root를 `<-`로 다시 결속한다. Definition과 별도 typed
semantic node를 사용한다.

## 표면과 의미

명시 등록 accepted-surface 집합과 closed multisurface는 하나의 semantic identity에 이르는
결정적 표면이다. RD-570에 따라 이름 뒤 `{...}`는 exception-only inflection이 아니며,
compiler는 자연어 어근·문법성·규칙성·파생 관계를 판정하지 않는다. morphology profile은
작성 편의일 뿐 admission·veto·hidden generation·fallback 권위가 없다. 후보 선택·pin identity
결정에서 source/declaration/import/registry order, hidden priority, fuzzy match, AI의 자연어
상식은 authority가 아니다.

이는 인자 식의 평가 순서를 없애는 규칙이 아니다. RD-242에 따라 인자 식은 원문
순서로 평가하고 호출마다 fresh callee pin을 결속한다. 이름형·조사형 결속은 pin
identity를 따르되 정본화가 평가 순서를 바꾸지 않는다. ordinary pin은 caller 저장
위치의 alias가 아니며 암묵 ref/out/inout이 없다. 따라서 효과가 있는 인자 식의
작성 순서를 바꾼 두 호출까지 무조건 같은 결과라고 주장하지 않는다.

## 비파급

source epoch 변경은 state, receipt, lock 또는 wire identity의 폐기를 뜻하지
않는다. 이 축은 별도 호환성 감사와 owner 결정이 필요하다.

## 수학 다중표면과 HoTT 방화벽

한국어·수학·ASCII source artifact는 다를 수 있지만 typed resolution 뒤 같은 의미면
semantic IR identity와 hash는 하나다. Proof R1 equality `=:=`를 HoTT higher path로
재해석하지 않는다. HoTT Path는 Type-valued 별도 identity family다.
정확한 의미 계산체계는 SPEC_PROOF_KERNEL_PROFILES_v0.9.0.md를 따른다.
RD-508의 Path·PropTrunc·composition 표면은 확정됐지만, 물리 schema·wire·kernel
제품 구현과 demand gate는 별도다. 의미 계산체계까지 일괄 OPEN으로 되돌리지 않는다.

RD-506의 `수식{E}`와 `세움{P}`는 각각 MathIR과 PropositionIR 구성이다.
구성만으로 수치평가·정리·증명이 실행되지 않는다. 계산은 명시적인 `풀기`가
담당하며, 명제 구성 성공을 증명 성공으로 표시하지 않는다.

## Product·Sum·Selection

ProductTypeDefinition, SumTypeDefinition, CallableDefinition은 하나의 Definition 경계를
공유하지만 typed category와 body shape는 다르다. Selection(Target, Selector)은
TargetType의 selector/result/failure/update 계약으로 해소하며 0개·복수 후보는
fail-closed한다.

## OwnedBranchIdentity 계보

고름씨의 `$갈래` 선언, `소유씨.$갈래(...)` 구성·참조, typed `에 따라` pattern은
같은 OwnedBranchIdentity를 가리킨다. 갈래 ordinal·source order·declaration order는
identity나 priority가 아니며 숫자 `[]`로 갈래를 해체하지 않는다.

## Selection 의미 행렬

Selection은 TargetType이 selector·result·failure·update 계약을 함께 소유한다.
동질 `(T)차림` index는 `T`, 0-based 반열림 range는 `(T)차림`를 내며 음수 뒤세기와
범위 밖 silent default를 금지한다. `(K,V)짝맞춤[K]`는 V를 내는 strict lookup이고,
missing key를 자동 option/default로 바꾸지 않는다. 안전 조회는 RD-476의 ordinary typed overload `찾기`가
`(V)있없` 계열을 반환한다. `찾기?`는 역사·명시적 이행 입력이지 영구 별칭이 아니다. `글[위치]`는 grapheme cluster 하나를 담은 `글`이며
code point·UTF unit·byte index로 조용히 바꾸지 않는다.

SelectionPlace와 `<-`는 기존 항목 교체·새 immutable 값·root rebinding으로 합성된다.
insert/delete/splice/중간 삽입/길이 변경 range replacement는 이 표면의 의미가 아니다.
`A[i,j]` MultiAxisSelection과 `A[i][j]` chained Selection은 별도 AST 의미다. 순서
없는 모음의 numeric `[]`는 금지한다. CollectionConstructor·PostfixSelection·Proof
projection은 canon roundtrip에서도 category를 보존한다.

## 호출별 ResultPlace

결과씨가 있는 named callable invocation은 `CallableDefinitionIdentity +
InvocationIdentity`에 결속된 하나의 ResultPlace를 가진다. ResultPlace는 Definition이나
전역 binding이 아니며 R1 source에서는 쓰기 전용이다. 순차 다시결속은 허용하지만
physical completion order나 병렬 child capture는 결과 authority가 아니다.

## 실패·종료·publish

typed value, execution disposition, failure family는 직교한다. 움직씨 ResultPlace는
local result candidate이고 atomic publish 성공 전에는 외부 성공 결과가 아니다.
실패·abort·publish 실패는 staged patch/outbox와 success candidate를 폐기한다.

## 조작 입력의 부분 승인 범위

RD691의 현재 효력은 다음 다섯 범위에 한정된다. 정확 물리 이름·단계·registry·V1 통합을 자동 확정하지 않는다.

1. 초기조건은 보이는 원문을 편집한 뒤 명시 재실행한다. 실행 중 조작은 typed recipe로 다음 마디 InputSnapshot에 전달하고 accepted commit 뒤 관찰/보기를 게시한다. 자동 재실행·DOM/renderer의 직접 세계 쓰기는 없다.
2. 수·좌표는 손실 없는 정수 저장과 대상·단위·범위에 결속한다. 물체 정체와 좌표값은 다르며 JS Number·표시 문자열·경로는 권위가 아니다. 특정 수폭·wire·범용 단위 환산은 승인하지 않는다.
3. 끌기 시작·이동·끝의 수락 사건 순서를 모두 보존한다. 임의 병합·누락·재정렬과 과거 마지막 위치 샘플링을 사용하지 않는다. overflow/무결성 실패를 성공·부분 게시로 바꾸지 않는다.
4. 조회 표면 수값·좌표값만 부분 승인 범위다. 단계 이름은 provisional이며 함수 서명·DefinitionIdentity·ParamPinIdentity·registry 배정을 자동 발급하지 않는다.
5. 기존 원·다각형·글과 기존 언어의 공용 그림 셈씨를 재사용한다. 새 전용 stdlib/Rust 우회를 허용하지 않는다. pan/zoom은 화면 local이며 world/tape/state_hash를 바꾸지 않는다. 공통 cursor/고정·상대 이력과 없는 이력 fail-closed를 보존한다.

기록 이름·머리표·V1 통합·물리 framing/tag/domain/bounds·진단/schema/선언 문법은 OPEN이다. 시험용 Set/Begin/Move/End 철자는 영구 wire가 아니다. 제품 landing·Alpha4 완료·Human·지원 전이를 주장하지 않는다.

## 현행 입력 관찰·저장 의미

같은 admitted snapshot과 같은 명시 조회 조건의 반복 조회는 같은 logical event 자료와 order를 본다. 조회는 consume/delete/ack/dequeue나 세계 변경이 아니다. 정상0건의 typed empty와 필수 구조 누락/손상의 실패는 다르다. reader-private iterator를 shared semantic cursor로 오인해 모두 금지하지 않는다. 기존 명시 message 소비·hand-off 의미는 유지한다(RD696).

합법 저장값/RecordFieldType 트리의 허용을 host 호출 스택 깊이로 결정하지 않는다. 기존 byte/count/section/artifact 한도 안에서 parse·validate·canonical encode/re-encode·거부된 부분 트리 파괴까지 stack-safe해야 한다. exact-fit은 검증을 계속하고 one-over는 할당·해석·순회·변경·게시 전 거부한다. 새 임의 depth 상한·tag·hash domain은 발급하지 않는다(RD694).

typed text/bool은 기존 Bool01/String04 ResourceValue 경로가 유일한 세계·저장·복원 공급이다. legacy resource_json과 내용 추론의 이중 경로·자동 migration은 없다. 빈 LE64(0) 구조 슬롯과 기존 codec·hash/domain은 보존한다. 이에 따른 영향 hash는 재계산 대상이며 동일성 보장을 추측하지 않는다(RD695).

## 자료·계산·보임의 현행 연결

조건별 함수는 선언 D 전체와 순서 없는 C/E 대응·정의 가능성·원 작성 admission을 보존한다. 자세한 의무와 빈 함수/반례는 조건별 계약을 따른다(RD706). 이름 있는 보임 결과의 직접 논리곱 생성은 전체 목표 검사 뒤에만 투영·후속 소비한다(RD707). 수학식 구성·명시 계산·가정 소비·보임 발급을 합치지 않는다.

<!-- REVIEWED_R21_BODY_END -->
