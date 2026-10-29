> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# source migration — Language SPEC v0.9.0 후속 후보

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

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

## v0.8.1에서 v0.9.0을 읽을 때

공개 문서 판올림은 사용자 source·저장 세계·정체·hash의 자동 변환이 아니다. 기존637개 결정 행은 같고 승격된 후속 의미가 겹치는 정확 범위만 적용한다. 부분 대체는 대체근거와_부분효력.md의 인용/범위로 읽으며 전체 RD를 삭제하지 않는다.

| 대상 | 필요한 처리 | 자동으로 하지 않는 것 |
|---|---|---|
| C2/C3 작성 | 정확 판본의 명시 유한 전개와 직접 C3 집합 동등성 검사 | 숨은 줄기/꼬리 생성·자동 최신판 |
| typed text/bool 저장 | 실제 source·typed carrier·custody와 legacy 의존 여부 확인 | resource_json 자동 추론·이중 경로·무조건 복원 |
| 조건별 함수 | D/C/E·binder·carrier·의무·작성 admission 보존 | 행순서 선택·자동 절단·빈 함수 추측 |
| 보임 생성 | 이름 있는 결과의 정확 목표·전체 검사 | ordinary Tuple 승격·미지원 문맥 확대 |
| 저장 재열기/이어 실행 | 원 continuation·next마디·권한·기록 유지 | 시작 재발동·새 seed/작품으로 위장 |
| 정확 비교 | 대상/사건·시점·완전성 공급 | 빠진 기록을 정상 empty·동등으로 바꾸기 |

문법 예제는 규격 설명용이다. 제품 admission·migration 도구·저장 변환은 고정 제품 근거 없이 완료라고 하지 않는다. 별도 도구가 필요하면 원본을 보존한 새 target으로 검증한다.

<!-- REVIEWED_R21_BODY_END -->
