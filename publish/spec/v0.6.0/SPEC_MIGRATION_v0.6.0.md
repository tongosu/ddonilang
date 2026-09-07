# source migration — Language SPEC v0.6.0 공개판

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> 내부 투영 기준: `v25.15.0 live current / RD-001~437`
> 공개 current: `v0.6.0`
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

## 입력 계약

migration은 최소 source bytes, source public-spec epoch, 선택한 migration
profile과 도구 identity를 봉인한다. 판본을 모르는 source를 경험적으로 해석하지
않는다.

## 처리 순서

```text
명시된 pre-v25 입력
→ 판본 검증
→ 결정적 변환 후보
→ 모호성 0 확인
→ v25 source 생성
→ 새 source identity로 parse·type·canon 재검증
→ receipt
```

UNKNOWN 또는 AMBIGUOUS면 실패한다. 옛 source를 지우거나 제자리 수정하지
않으며 migration 산출물은 별도 파일로 만든다. exact migration schema와 제품은
OPEN이다.

## v25.2 이후 clean-break

- 이름 있는 callable의 implicit tail result는 ResultPlace 결속으로 바꾼다.
- `~지 못하면` recovery surface는 `해보고 안되면` family로 옮긴다.
- `값 아니면 되돌림`은 영구 alias 없이 retire하며 typed failure 결과를 ResultPlace에
  기록하고 필요한 경우 `마침.`을 사용한다.
- ordinary optional `T?`는 source version과 옛 의미가 유일할 때만 `(T)있없`으로
  migration한다.
- UNKNOWN·AMBIGUOUS인 입력은 자동 변환하지 않는다.
