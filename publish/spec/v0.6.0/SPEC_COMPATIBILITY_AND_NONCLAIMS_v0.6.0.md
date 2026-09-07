# 호환성·비주장 — Language SPEC v0.6.0 공개판

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> 내부 투영 기준: `v25.15.0 live current / RD-001~437`
> 공개 current: `v0.6.0`
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
