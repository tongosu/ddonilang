> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 호환성·비주장 — Language SPEC v0.9.0 후속 후보

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

## source clean break

- pre-v25 source 표면은 일반 v25 parser의 영구 compatibility alias가 아니다.
- 옛 source는 명시적인 migration mode의 입력으로만 처리한다.
- 옛 표면을 현재 의미로 자동 추측하거나 조용히 고치지 않는다.
- public SPEC v0.5.0과 이전판, legacy·negative·compatibility corpus는 역사로
  제자리 보존한다.

## 보존되는 별도 축

이 source clean break만으로 state·receipt·lock·wire 호환성을 폐기하지 않는다.
해당 축은 별도 감사 전까지 OPEN이다.

## 비주장

이 공개판의 문서 current 승격은 parser/runtime/formatter, migration 제품, native/WASM/Workbench
parity, D-PACK oracle, Human PASS, 외부 사용자 성공 또는 배포 완료를 뜻하지
않는다.

## R2 추가 비주장

이 공개판은 Product/Sum/Selection/Tensor와 PathIR·Proof의 닫힌 공개 semantic contract를
투영하지만 exact physical parser·AST·wire·diagnostic·runtime·proof kernel encoding,
safe lookup exact 이름, executable oracle 또는 제품 parity를 승인하지 않는다. 연구
surface는 미래 grammar compatibility freeze가 아니다.

## R3 교정 후에도 OPEN인 경계

R5는 Tensor rank·static slice·immutable value와 failure taxonomy의 닫힌 의미를 공개
본문에 투영한다. 계속 OPEN인 것은 safe lookup provider의 최종 한국어 이름,
deferred Tensor feature, physical parser·AST enum·wire·runtime·diagnostic·proof kernel,
executable oracle과 제품 parity다.


## v25.137 current-view successor precedence

- RD-608은 official zero-instance full-headless validity에 한해 RD-555의 bounded nonclaim을 additive하게 대체한다.
- RD-610은 PG-CLI-05 construction과 typed sum-match에서 RD-438·RD-501의 expected-owner bare branch 허용을 함께 좁힌다. TypeIdentity qualifier가 필수다.
- RD-618은 RD-541 bracket profile 둘을 보존하지만 옛 callable spelling과 public pin `끝`은 history-only이며 permanent alias가 아니다.
- 역사 행은 trace에 남고 current meaning은 위 successor를 따른다.


## v25.137 successor boundary

- RD-643/656/658/682/684 add public semantic surfaces without turning their physical
  storage, private ABI or product custody into public wire promises.
- RD-656 keeps Bytes distinct from String and numeric List, uses strict UTF-8 and explicit
  Unicode 17.0.0 NFC decode, and admits only `UTF8바이트열로` / `UTF8NFC글로` with no aliases.
- RD-658 makes `{ 조건 }인 동안 { ... }.` the ordinary canonical form and preserves
  `(조건)인 동안 { ... }.` as the separate hook owner; legacy `인것 동안` forms are not aliases.
- Complete-row bindings remain status evidence only. They do not create public support,
  readiness, release or Human PASS.

## v25.151 현행 의미·이관 범위

- RD694는 기존 저장 byte/count/section 한도와 codec/hash를 보존하며 host stack 깊이로 새 의미 상한을 만들지 않는다.
- RD695는 typed text/bool clean-break의 legacy resource_json 추론·dual path·자동 migration을 허용하지 않는다. typed V1 복원은 보존하되 legacy 의존·불완전 복원·continuation 없는 resume를 거부한다. 영향 hash의 재계산과 저장 포맷 전체 교체는 다르다.
- RD696은 조회의 비소비를 명시하고 기존 RD283/284의 명시 message 소비를 보존한다.
- RD706/707/708/709의 제한 문맥·가정·작성 admission·lifecycle를 지우는 호환 변환을 만들지 않는다.
- 화살표 동등성·선언된 나·일본어 기획·A{A}·복구 RHS의 미승격 부분은 비규범 후속안이다. 제품의 수용이나 교재 예제가 규범 승격 권위가 아니다.
- 초기 RD001~053의 역사 존재와 개별 현행 계승은 구분한다. UNKNOWN을 복원·허용·폐기 근거로 쓰지 않는다.

과거 공개판과 실패 후보는 보존한다. 이 후보의 최종 공개 SPEC CURRENT 승격과 GitHub 게시·제품 지원 확대는 별도 승인이다.

<!-- REVIEWED_R21_BODY_END -->
