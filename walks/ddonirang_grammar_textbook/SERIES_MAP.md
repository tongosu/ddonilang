# 또니랑 문법·사용 총서 — SERIES_MAP 초안

문서 상태: draft (r2)  
참조 SSOT: v24.12.0  
용도: 사람에게 보이는 총서 번호와 내부 anchor / pack family / 집필 원고를 연결한다.
또한 first-run/positioning/product path 의 교차축을 가볍게 표시한다.

---

## 1. 총서 구조

### 본문 9권
1. 시작과 초급
2. 차림과 자료 흐름
3. 실행과 시뮬레이션
4. 고급 구조
5. 판과 마당
6. 보개와 거울
7. 격자와 입력
8. 전문과 AI 경계
9. web smoke와 pack 실습실

### 참조 3권
- REF-01 정본 문법 사전
- REF-02 호환·legacy·이행 가이드
- REF-03 총서·pack·status 지도

### 예약 1권
- 10권 교사용 운영노트

---

## 1.5 first-run 교차축

canonical first-run path `Blocky -> grid2d -> 보개 -> 거울` 은 단일 권에 갇히지 않는다.

- 1권: 첫 문장 / 값 / 출력 / state_hash 첫 감각
- 3권: `(시작)할때` / `(매마디)마다` / grid2d 실행 감각
- 6권: 거울 / replay / timeline 첫 감각
- 9권: web smoke / pack 실습실 연결

## 2. 내부 anchor ↔ 사람용 총서 번호

| internal anchor | 사람용 총서 | 메모 |
|---|---|---|
| `01_기초.md` | 1권 일부 anchor | 초급 노출 계층 anchor |
| `01_시작과_초급.md` | 1권 일부 anchor | 시작 seed / pack 연결선 |
| `02_중급.md` | 2권 일부 anchor | 현재 중급 skeleton |
| `03_고급.md` | 4권 anchor | 사람용 총서에서는 고급 구조권 |
| `04_전문.md` | 8권 anchor | 전문과 AI 경계 |
| `05_web_smoke_lab.md` | 9권 anchor | web smoke / pack 실습 |

---

## 3. 집필 원고 ↔ 사람용 총서 번호

| 원고 source | 사람용 총서 | 범위 |
|---|---|---|
| `docs/context/manuscripts/ddonirang_series/01_시작과_초급/00_권_합본.md` | 1권 | 전체 |
| `docs/context/manuscripts/ddonirang_series/02_차림과_자료_흐름/00_권_합본.md` | 2권 | 전체 |
| `ddonirang_vol3_lesson01_v1.zip` | 3권 | lesson01 ~ lesson16 |
| `ddonirang_vol3_lesson01_v1.zip` | 4권 | lesson17 ~ lesson44 |
| `ddonirang_vol3_lesson01_v1.zip` | 부록/운영노트 후보 | lesson45 ~ lesson47 |
| lifecycle/거울/grid/input SSOT | 5~7권 | 신규 집필 필요 |
| 전문/web smoke skeleton | 8~9권 | 확장 집필 필요 |

---

## 4. pack family ↔ 사람용 총서 번호

| pack family | 현재 이름 | 사람용 총서 해석 |
|---|---|---|
| Vol1 | `edu_ddn_vol1_*` | 1권 시작과 초급 |
| Vol2 | `edu_ddn_vol2_*` | 3권 실행과 시뮬레이션과 강하게 대응 |
| Vol3 | `edu_ddn_vol3_*` | 4권 고급 구조와 강하게 대응 |
| Vol4 | `edu_ddn_web_smoke_*` | 9권 web smoke와 pack 실습실 |

메모:
- pack family rename은 즉시 하지 않는다.
- 먼저 alias map을 문서로 고정한다.
- 사람이 읽는 총서 번호와 internal pack family 이름은 일정 기간 공존한다.
- **REF-03 alias map이 확정되기 전까지 pack family 이름과 사람용 총서 번호는 일치하지 않을 수 있다.**

---

## 5. 빠진 축 점검표

| 축 | 총서 배치 | 상태 메모 |
|---|---|---|
| `판`, `마당`, `시작하기`, `넘어가기`, `모두다시` | 5권 | lifecycle/sim/game core |
| `기억 {}` / `갈림 {}` | 5권 | docs-first / proposal 상태를 권 서문에서 명시 |
| `거울` record/query/timeline | 6권 | view/replay/query 경계 |
| `std_grid`, `std_input_map`, `grid2d-space` | 7권 | grid/input/game core |
| proof / capability / 슬기 / 배움틀 / model artifact minimum | 8권 | 전문 설명층 + 최소 사용 경계 |
| parser / canon / runtime / view / D-PACK | 9권 | smoke/pack 실습축 |
| legacy root surface / removed surface / compat alias | REF-02 | teaching bridge / 이행 가이드 |
| 말힘누리 story-tech 축 (`임자/맞물림씨/기억/세움/갈림`) | 4권 + 5권 + 8권 교차 / REF-03 추적 | 단일 권으로 확정하지 않고 교차축으로 관리 |

---

## 6. 말힘누리 교차축 임시 배치 원칙

말힘누리 축은 현재 총서에서 단일 권으로 바로 고정하지 않는다.
우선은 아래처럼 교차 배치한다.

- `임자`, `세움`, 구조적 사건 흐름의 바닥은 **4권**
- `기억`, `갈림`, lifecycle/장면 진행과 맞닿는 부분은 **5권**
- story-tech가 current-line인지 docs-first인지 읽는 경계는 **8권**
- 최종 번호/독립 권 여부는 **REF-03**에서 추적한다.

이 원칙은 “말힘누리 축이 총서 어디에도 없다”는 상태를 피하면서도,
아직 SSOT current-line으로 닫히지 않은 범위를 과장하지 않기 위한 임시 배치다.

---

## 7. 편집상 주의

- source-of-truth는 `docs/ssot/**` 이다.
- `docs/context/all/**` 합본은 working/export aggregate다.
- 교재 총서는 SSOT 전체 운영문서를 복제하지 않는다.
- pack/checker/CI evidence는 총서 본문이 아니라 status/참조권에서 다룬다.
- 5권, 8권, 말힘누리 교차축은 서문마다 current-line / docs-first / deferred를 반복 표기한다.


## 8. REF-03 연결

- 상세 alias map / current-line / follow-on minimum / docs-first / deferred 대응표는 `REF_03_총서_pack_status_지도.md`가 맡는다.
- pack family 물리 rename은 즉시 하지 않는다.
- 사람용 총서 번호와 internal anchor/current pack family 이름은 일정 기간 공존한다.


## v24.12.0 note

- roadmap V2 / platform ecosystem / JOJO economy / multilang export proposal 은 총서 번호/anchor를 직접 바꾸지 않는다.
- teaching/publication overlay 와 current-line/pending boundary 를 가르는 원칙은 그대로 유지한다.
