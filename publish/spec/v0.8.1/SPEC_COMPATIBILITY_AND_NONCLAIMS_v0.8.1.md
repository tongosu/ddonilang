# 호환성·비주장 — Language SPEC v0.8.1 공개판

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`
> 내부 투영 기준: `v25.137.0 live source / RD-001~690`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

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
