> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 실행 lifecycle·정확 비교 — 현행 계약

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

## root 훅 발동과 원문 순서

| 상황 | 시작 | 매마디와 공개 경계 |
|---|---|---|
| 새 의미 실행 첫 실제 마디 | 해당 | 첫 마디 포함 해당, 같은 진입 단위 root 원문 순서 |
| 같은 실행 정상 이어가기 | 재발동 없음 | 이어지는 실제 마디에 적용 |
| 저장 재열기·host 재시작 | 그 자체로 재발동 없음 | admission된 continuation과 원 next마디를 따름 |
| 같은 실행 새 record epoch | 새 시작권 없음 | 기록 epoch와 의미 실행을 구분 |
| 명시 새 실행 | 새 실행 첫 마디에 해당 | 이전 세계에 수정 원문 자동 적용 없음 |
| 원 seed부터 격리 재생 | 원 시작 계산 재현 | 원세계/외부 effect 추가 공개 없음 |
| receipt 멱등 조회·복구 게시 확인 | body 재실행 아님 | 기존 durability·custody 확인을 따름 |

시작은 무조건 먼저가 아니라 첫 마디에만 적용된다. 시작을 앞에 두는 초기화 작성은 권장이다. 뒤에 썼다는 이유로 오류·hoist·종류별 phase·body dedup을 만들지 않는다. 첫 마디 후보 실패 시 허용된 재평가와 이미 성공한 시작의 중복 공개를 구분한다. raw tick0·빈 세계·host bool만으로 발동 정체를 발급하지 않는다. 다른 module/provider/worker 전체의 전역 순서로 확대하지 않는다.

시작 body 완료와 마디 전체 성공 공개는 다르다. 뒤 작업·타입·권한·출력·충돌 검증 실패면 해당 마디의 부분 세계·성공 출력·receipt 공개는0이고 이전 확정 prefix를 보존한다. typed 실패값을 자동 hard failure로 바꾸지 않고 자동 재시도·새 liveness·복구권을 만들지 않는다(RD709).

## 정확 비교·되감기

원본 보기, 같은 봉인 입력 재생, 선언 입력 하나를 바꾼 처음부터 실험은 별도다. 비교는 실제형·단위·판본에 맞는 정확 동등성을 먼저 사용하며 π를 긴 소수나 유리수로 위장하거나 임의 오차를 넣지 않는다. 근사 오차 정책은 후속이다(RD708).

관찰의 공급/검증·명시 대상/사건 대응·논리 공개 시점과 구간 양끝 [a,b]·기록 완전성을 결속한다. 필수 기록·누락·잘림·종료·저장 확정을 따로 검증하고 prefix 첫 차이를 보고한다. 비교 기준·실행 계약·원 실행 상태·목적별 해석을 구분한다. 원 실패/RecoveryRequired를 비교 성공으로 덮지 않는다.

## 기술·제품 경계

실제 lifecycle owner/중복 마디 증거·root/제한 임자 lowering 순서·continuation carrier·Native/CLI/WASM/Chrome parity는 기술 OPEN이다. RD709 12벡터와 RD708 19벡터는 PLANNED_NOT_ORACLE/NOT_RUN을 보존한다. 비교기의 실제 공급·adapter·issuer·저장 schema/wire도 OPEN이다. lifecycle·정확 비교의 문서 승격이 제품 착수나 자동 복구 승인은 아니다.

<!-- REVIEWED_R21_BODY_END -->
