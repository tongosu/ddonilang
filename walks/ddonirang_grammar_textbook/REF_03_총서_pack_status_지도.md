
# REF-03 — 총서·pack·status 지도

참조 SSOT: v24.10.0  
상태: draft / reference / alias-map

---

## 목적
사람용 총서 번호, landed internal anchor, pack family, working manuscript, status 표기를 **한 표로 함께 읽기 위한 지도**다.

- source-of-truth는 `docs/ssot/**` 이다.
- 이 문서는 teaching/publication overlay reference 이다.
- pack family 물리 rename은 즉시 하지 않는다.

---

## A. 총서 ↔ internal anchor ↔ manuscript

| 사람용 총서 | internal anchor | working manuscript | status |
|---|---|---|---|
| 1권 시작과 초급 | `01_기초.md`, `01_시작과_초급.md` | `docs/context/manuscripts/ddonirang_series/01_시작과_초급/00_권_합본.md` | current-line teaching overlay |
| 2권 차림과 자료 흐름 | `02_중급.md` 일부 reference | `docs/context/manuscripts/ddonirang_series/02_차림과_자료_흐름/00_권_합본.md` | human-facing series overlay |
| 3권 실행과 시뮬레이션 | `02_중급.md` 후반 / Vol2 pack cluster | 추후 집필 | follow-on minimum |
| 4권 고급 구조 | `03_고급.md` | 추후 집필 | current-line / follow-on mixed |
| 5권 판과 마당 | lifecycle/sim/game core | 추후 집필 | docs-first / exact contract |
| 6권 보개와 거울 | 거울/replay/query/timeline | 추후 집필 | minimum_landed + follow-on |
| 7권 격자와 입력 | `std_grid`, `std_input_map`, `grid2d-space` | 추후 집필 | current-line / follow-on mixed |
| 8권 전문과 AI 경계 | `04_전문.md` | 추후 집필 | docs-first / expert boundary |
| 9권 web smoke와 pack 실습실 | `05_web_smoke_lab.md` | 추후 집필 | current-line teaching lab |

## B. 총서 ↔ pack family

| 사람용 총서 | pack family | 메모 |
|---|---|---|
| 1권 | `edu_ddn_vol1_*` | physical rename 하지 않음 |
| 2권 | (직접 1:1 없음) | manuscript 중심, Vol2/Vol3 cluster와 일부 교차 |
| 3권 | `edu_ddn_vol2_*` | 실행/시뮬레이션 cluster 와 강하게 대응 |
| 4권 | `edu_ddn_vol3_*` | 고급 구조 cluster 와 강하게 대응 |
| 9권 | `edu_ddn_web_smoke_*` | web smoke / pack 실습 |

## C. status legend

| 표기 | 뜻 |
|---|---|
| current-line | 현재 정본/도구/pack 근거가 있는 축 |
| follow-on minimum | 다음 묶음으로 이어질 최소 바닥 |
| docs-first | 설명/설계는 있으나 runtime claim은 아직 없는 축 |
| deferred | 장기 이월 축 |
| teaching/publication overlay | 사람에게 보이는 집필/출판 구조. internal anchor current semantics 를 직접 바꾸지 않음 |

## D. 말힘누리 교차축

| 축 | 사람용 총서 배치 | 메모 |
|---|---|---|
| `임자 / 세움` | 4권 | 구조/규범 바닥 |
| `기억 / 갈림 / lifecycle` | 5권 | docs-first / exact contract |
| current-line / docs-first / AI 경계 | 8권 | 전문 설명층 |

말힘누리 축은 단일 권으로 성급히 고정하지 않는다.
