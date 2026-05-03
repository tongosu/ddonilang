# 또니랑 문법 교재 (ddonirang_grammar_textbook)

참조 SSOT: v24.12.0  
참조 proposal: `PROPOSAL_DDONIRANG_GRAMMAR_USAGE_SERIES_9P3_V1_20260424_r2.md`  
최종 갱신: 2026-04-26 (v24.12.0 local refresh)  
상태: README 교체안 초안 (r2)

---

## 개요

이 교재는 또니랑(DDonirang) DSL의 문법을 두 층으로 정리한다.

1. **내부 anchor 파일**
   - `exposure_level` 기준으로 landed 된 skeleton
   - 현재 파일명: `01_기초`, `01_시작과_초급`, `02_중급`, `03_고급`, `04_전문`, `05_web_smoke_lab`

2. **사람에게 보이는 총서**
   - 또니랑의 문법과 사용을 단계적으로 배우기 위한 집필/출판 구조
   - 아래 1권~9권은 이 사람용 총서 번호다.

---

## 바깥 설명 한 줄

또니랑은 **웹 앱 CRUD 중심 언어가 아니라 한국어 네이티브 세계 모델링 언어**다.
수학·물리·경제·시뮬레이션·RL·설명 가능한 실행을 같은 코어 위에서 다룬다.

## 첫 사용자 canonical path

처음 또니랑을 여는 사람에게 권장하는 기본 경로는 아래다.

- **Blocky** 에서 한 동작 바꾸기
- 같은 예제를 **grid2d** 에서 즉시 실행
- 같은 상태를 **보개** 로 보기 전환
- **거울** 에서 tick 을 되감아 보기

이 경로는 제품 path 이자 teaching path 다.

## 핵심 사용자 3종

- **배움 사용자**: Blocky, textbook, graph/grid2d, 보개, 거울을 먼저 만난다.
- **만드는 사용자**: grid2d / 보개 / timeline / local registry minimum 을 먼저 만난다.
- **연구 사용자**: NuriGym / replay / state_hash / evidence 를 먼저 만난다.

## 핵심 원칙

- source-of-truth는 항상 `docs/ssot/**` 이다.
- 현재 anchor 파일명과 사람용 총서 번호는 **1:1 대응이 아닐 수 있다**.
- 총서 번호를 바꾼다고 pack family 이름을 즉시 바꾸지는 않는다.
- closure 판정은 계속 문서 제목이 아니라 **pack/checker/CI evidence**를 따른다.
- `AGE`와 `exposure_level`은 서로 다른 축이다.
- 특히 5권~9권은 권마다 **current-line / follow-on minimum / docs-first / deferred** 상태를 권 서문에서 숨기지 않는다.

---

## 노출 계층 4단계 (internal anchor 유지)

| 계층 | 기준 | 대상 |
|---|---|---|
| **초급** | `채비`+훅만으로 완결 실행 가능 | 중등 교육, 입문 |
| **중급** | 시뮬레이션/제어 구조 추가 | 대학교 입문, 시뮬레이션 제작 |
| **고급** | 계약/원자성/임자/메시지 | RL 환경, 멀티에이전트 |
| **전문** | NuriGym / 증명 / 슬기 / capability | 연구자, 플랫폼 개발 |

---

## 사람에게 보이는 총서 구조 (series map)

### 총서 본권 9권

| 총서 | 권명 | 핵심 범위 | 주 source / anchor |
|---|---|---|---|
| 1권 | 시작과 초급 | 첫 문장, 값, 상태, 출력, state_hash 첫 감각 | `01_시작과_초급.md`, `01_기초.md`, 1권 집필본 |
| 2권 | 차림과 자료 흐름 | 차림, 범위, 순회, 고르기, 자료 처리 | 2권 집필본 중심 |
| 3권 | 실행과 시뮬레이션 | `(시작)할때`, `(매마디)마다`, `채비 {}`, `수식{}`, `풀기`, `매김 {}` | 3권 working 원고 전반부, current pack Vol2 cluster |
| 4권 | 고급 구조 | 임자, 알림씨, 상태머신, `X에 따라`, 계약, 덩이, 복귀/재개 | `03_고급.md`, 3권 working 원고 후반부 |
| 5권 | 판과 마당 | `판`, `마당`, `시작하기`, `넘어가기`, `모두다시`, resident `:reset`, `기억 {}`, `갈림 {}` | lifecycle/sim/game core + story-tech follow-on 일부 |
| 6권 | 보개와 거울 | `보임`, 출력/상태 경계, `거울`, replay/query/timeline | 거울 boundary + 1권 후반 감각 |
| 7권 | 격자와 입력 | `std_grid`, `std_input_map`, `grid2d-space`, 게임/누리 입력 패턴 | grid/input core |
| 8권 | 전문과 AI 경계 | proof, capability gate, 슬기, 배움틀, model artifact minimum | `04_전문.md` + 전문 경계 문서군 |
| 9권 | web smoke와 pack 실습실 | parser, canon, runtime, view, `flat_json`, `maegim_plan`, `alrim_plan`, 작은 D-PACK | `05_web_smoke_lab.md`, textbook seed index |

### 참조권 3종

| 구분 | 이름 | 역할 |
|---|---|---|
| REF-01 | 정본 문법 사전 | surface, canonical 이름, status 표기, 경계 요약 |
| REF-02 | 호환·legacy·이행 가이드 | 예전 표면, 제거 표면, 옮겨 쓰기 규칙, teaching bridge |
| REF-03 | 총서·pack·status 지도 | 사람용 총서 번호 ↔ current anchor ↔ pack family ↔ status 대응표 |

### 예약권

| 구분 | 이름 | 역할 |
|---|---|---|
| 10권(예약) | 교사용 운영노트 | 차시 운영, 학습지, 답안, 평가, pack 연결 |

---

## 권별 공식 목차 요약

### 1권 — 시작과 초급
부제: 첫 문장부터 state_hash 감각까지

- 1부. 첫 문장과 값
- 2부. 셈과 판단
- 3부. 반복과 작은 흐름
- 4부. 또니랑다운 감각

설명:
- 또니랑을 처음 읽는 학습자를 위한 입문권이다.
- 작은 예제, 읽기/쓰기/실행/바꿔보기, 출력과 상태의 구분을 중심으로 한다.

### 2권 — 차림과 자료 흐름
부제: 차림, 범위, 고르기, 다시 쓰기

- 1부. 차림 다시 보기
- 2부. 범위와 계산
- 3부. 결과의 모양 읽기
- 4부. 골라내기와 바꾸기
- 5부. 여러 단계 자료 흐름

설명:
- 차림과 범위를 따라 데이터를 읽고 바꾸는 감각을 기른다.
- 표/그래프/비교/조건을 포함한 자료 흐름 읽기를 다룬다.

### 3권 — 실행과 시뮬레이션
부제: 움직이는 또니랑, 훅, 채비, 수식, 풀기, 매김

- 1부. 움직이는 또니랑의 첫 감각
- 2부. 채비와 준비
- 3부. 수식과 풀기
- 4부. 매김과 조절
- 5부. 작은 시뮬레이션
- 6부. 3권 마무리

설명:
- 시간과 훅을 도입해 “움직이는 코드”를 읽게 한다.
- 수식 저장/평가, 매김, 작은 물리·수학 실습을 포함한다.

### 4권 — 고급 구조
부제: 임자, 알림씨, 상태머신, 계약, 덩이

- 1부. 임자와 자기참조
- 2부. 알림과 사건 흐름
- 3부. 상태 전이와 상태머신
- 4부. 값의 모양과 분기
- 5부. 계약, 물림, 덩이
- 6부. 복합 흐름, 복귀, 재개

설명:
- 또니랑의 고급 제어와 구조적 사건 흐름을 다룬다.
- RL 환경, 멀티에이전트, 복합 상태 흐름으로 이어지는 바닥을 다진다.

### 5권 — 판과 마당
부제: lifecycle, reset, episode, scenario

- 1부. `판`과 `마당`의 뜻
- 2부. `시작하기` / `넘어가기` / `불러오기`
- 3부. `마당다시` / `판다시` / `누리다시` / `보개다시` / `모두다시`
- 4부. resident `:reset`와 episode reset
- 5부. `기억 {}` / `갈림 {}` / 작은 lifecycle 실습

설명:
- 시뮬레이션/게임/누리 실행의 공통 lifecycle을 다룬다.
- 한 번의 시도, 장면 전환, 다시 시작의 의미를 구분한다.
- **이 권의 일부 범위는 docs-first proposal 상태이며, 실제 집필 범위는 SSOT 승격 후 확정한다.**
- 특히 `기억 {}` / `갈림 {}` 과 말힘누리 story-tech 결합 축은 current-line 완료가 아니라 follow-on/draft로 표기한다.

### 6권 — 보개와 거울
부제: 보임, 보개, replay, query, timeline

- 1부. `보임`, `보여주기`, `보개` 구분
- 2부. 출력·상태·`state_hash` 경계
- 3부. 거울의 설정층 / 기록층 / UI층
- 4부. replay / query / seek / timeline
- 5부. 검증과 관찰의 연결

설명:
- 무엇이 상태이고 무엇이 보기/기록/진단인지 분리해 이해하게 한다.
- `거울`은 디버그 꾸밈이 아니라 기록과 재생의 경계로 다룬다.

### 7권 — 격자와 입력
부제: std_grid, std_input_map, grid2d-space

- 1부. `std_grid`
- 2부. `std_input_map`
- 3부. `grid2d`와 `space2d`
- 4부. console-grid에서 web2d까지
- 5부. 퍼즐/격자형 실습

설명:
- 격자 상태와 입력 사상을 canonical world/state 기준으로 다룬다.
- 퍼즐, 테트리스, 로그라이크형 사용 패턴의 공통 바닥을 정리한다.

### 8권 — 전문과 AI 경계
부제: proof, capability, 슬기, 배움틀

- 1부. proof 축 읽기
- 2부. capability gate 3장부
- 3부. 슬기 = 모델 이름이 아니라 의도 경계
- 4부. 배움틀 = 공방/toolchain 층
- 5부. infer-only model artifact minimum
- 6부. current-line / docs-first / deferred 읽기

설명:
- 전문권은 “새 마법문 모음집”이 아니라 경계 설명서다.
- 어디까지가 current line이고 어디서부터 follow-on인지 구분해 읽는 법을 다룬다.

### 9권 — web smoke와 pack 실습실
부제: parser, canon, runtime, view, pack

- 1부. parser / canon / runtime / view
- 2부. `core_lang -> seamgrim -> profile gate`
- 3부. `flat_json`, `maegim_plan`, `alrim_plan`
- 4부. 작은 D-PACK 작성
- 5부. golden, checker, evidence로 읽는 법

설명:
- web smoke는 예쁜 데모가 아니라 경계 확인 실습으로 다룬다.
- parser/canon/runtime/view와 pack evidence를 함께 읽게 한다.

---

## 파일 목록 (internal anchor)

| 파일 | 내용 |
|---|---|
| `01_기초.md` | 초급 계층 — 선언, 대입, 기본 훅, 출력 |
| `01_시작과_초급.md` | 시작 seed — 초급/walk/pack 연결선 |
| `02_중급.md` | 중급 계층 — 조건, 반복, 매김, 흐름씨, 될때/인 동안 |
| `03_고급.md` | 고급 계층 — 임자, 알림씨, 상태머신, X에 따라, 계약 |
| `04_전문.md` | 전문 계층 — proof/capability/슬기/배움틀 설명 shell |
| `05_web_smoke_lab.md` | web smoke 실습실 — parser/canon/runtime/view 경계 |

---

## 편집 메모

1. 현재 anchor 파일은 유지한다.
   - `01_기초`~`05_web_smoke_lab`는 내부 정렬용 landed skeleton이다.
   - 사람용 총서는 이 위에 얹는 집필/출판 구조다.

2. 1권과 2권은 기존 집필본을 우선한다.
   - 이미 권 단위 본문 흐름이 존재하므로, 이 둘은 새로 찢기보다 정리/보강이 우선이다.

3. 3권과 4권은 현재 3권 working 원고를 둘로 나누어 읽는다.
   - 전반부는 실행/시뮬레이션
   - 후반부는 고급 구조

4. 5권~7권은 신규 집필이 필요하다.
   - 다만 범위와 어휘는 이미 SSOT에 흩어져 있으므로 새 문법을 발명하지 않는다.
   - 5권은 특히 story-tech/lifecycle 교차축이 있어 서문에서 docs-first/follow-on 표시를 반복한다.

5. 8권과 9권은 현재 `04_전문.md`, `05_web_smoke_lab.md`를 확장 집필하는 방향을 따른다.

6. pack family 이름은 즉시 바꾸지 않는다.
   - 먼저 사람용 총서 번호 ↔ internal pack family 대응표를 `REF-03`에 고정한다.

---

## status 표기 규칙

| 표기 | 뜻 |
|---|---|
| current-line | 현재 정본/도구/pack 근거가 있는 축 |
| follow-on minimum | 다음 묶음으로 이어질 최소 바닥 |
| docs-first | 설명/설계는 있으나 runtime claim은 아직 없는 축 |
| deferred | 장기 이월 축 |

총서 문안은 위 표기를 숨기지 않는다.
특히 전문권, 거울권, lifecycle권, pack권은 “무엇이 이미 닫혔고 무엇이 아직 제안인지”를 각 장 첫머리에서 함께 밝힌다.


## working manuscript 연결선

| manuscript | 경로 | 의미 |
|---|---|---|
| 1권 current-line manuscript v2 | `docs/context/manuscripts/ddonirang_series/01_시작과_초급/00_권_합본.md` | current-line 정렬 입문권 초안 |
| 2권 series manuscript v1 | `docs/context/manuscripts/ddonirang_series/02_차림과_자료_흐름/00_권_합본.md` | 사람용 총서 2권 권 단위 합본 초안 |
| REF-03 alias map | `REF_03_총서_pack_status_지도.md` | 총서 번호 ↔ anchor ↔ pack family ↔ status 지도 |

이 세 문서는 **teaching/publication overlay working line** 이며, landed internal anchor(`01_기초`~`05_web_smoke_lab`)의 current-line semantics 를 직접 대체하지 않는다.


## v24.12.0 note

- roadmap V2 / platform ecosystem / JOJO economy / multilang export line 은 총서 번호를 직접 바꾸지 않는다.
- 이 교재는 계속 사람용 총서 overlay 와 internal anchor / pack family alias map 을 함께 유지한다.
