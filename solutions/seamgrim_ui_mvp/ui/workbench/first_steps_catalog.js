// Owner 2026-10-05: definition-centered ten-lesson alpha1 course.
// V3 uses the same catalog and real root/main files; notebook math is explicit.
export const FIRST_STEPS_COURSE_MANIFEST = {
  "purpose": "수업 검증용 목록이며 제품 실행 입력 형식이 아님",
  "profile": "v1-core-v25",
  "cases": [
    {
      "id": "01A",
      "title": "첫계산",
      "file": "예제/01A_첫계산.ddn",
      "expected": [
        "7"
      ],
      "reason": "3+4=7"
    },
    {
      "id": "01B",
      "title": "글과수",
      "file": "예제/01B_글과수.ddn",
      "expected": [
        "안녕",
        "24"
      ],
      "reason": "글은 그대로, 8*3=24"
    },
    {
      "id": "01C",
      "title": "이름붙이기",
      "file": "예제/01C_이름붙이기.ddn",
      "expected": [
        "2400"
      ],
      "reason": "800*3=2400"
    },
    {
      "id": "01D",
      "title": "결속바꾸기",
      "file": "예제/01D_결속바꾸기.ddn",
      "expected": [
        "3200"
      ],
      "reason": "보통 결속을 3에서4로 변경;800*4=3200, := 이름을 다시 결속하지 않는다"
    },
    {
      "id": "02A",
      "title": "두배정의",
      "file": "예제/02A_두배정의.ddn",
      "expected": [
        "14"
      ],
      "reason": "7*2=14"
    },
    {
      "id": "두배와나누기_조사중첩_1",
      "title": "두배를 구해 나누기",
      "file": "예제/두배와나누기.또니",
      "expected": [
        "14",
        "14",
        "14",
        "2"
      ],
      "reason": "같은 셈씨를 조사형으로 부르고 두배 결과를 나눌수에 전달;12/(3*2)=2",
      "project_id": "두배와나누기_조사중첩_1",
      "nativeStandaloneSource": true,
      "project_title": "두배를 구해 나누기",
      "source_path": "두배와나누기.또니",
      "asset_root": "../../first_steps/두배와나누기_조사중첩_1/",
      "student_doc": "학생용.html",
      "teacher_doc": "교사용.html"
    },
    {
      "id": "02C",
      "title": "두받는자리",
      "file": "예제/02C_두받는자리.ddn",
      "expected": [
        "20"
      ],
      "reason": "8*5/2=20"
    },
    {
      "id": "02D",
      "title": "이름핀",
      "file": "예제/02D_이름핀.ddn",
      "expected": [
        "24"
      ],
      "reason": "핀 순서를 바꿔도 8*6/2=24"
    },
    {
      "id": "03A",
      "title": "실행정의",
      "file": "예제/03A_실행정의.ddn",
      "expected": [
        "14"
      ],
      "reason": "실행을 명시 호출;7*2=14"
    },
    {
      "id": "03B",
      "title": "직접만든움직씨",
      "file": "예제/03B_직접만든움직씨.ddn",
      "expected": [
        "7",
        "9"
      ],
      "reason": "기록 동작 두 호출 순서"
    },
    {
      "id": "03C",
      "title": "계산과동작분리",
      "file": "예제/03C_계산과동작분리.ddn",
      "expected": [
        "14"
      ],
      "reason": "셈씨가 계산하고 움직씨가 출력"
    },
    {
      "id": "03D",
      "title": "계산만남기기",
      "file": "예제/03D_계산만남기기.ddn",
      "expected": [],
      "reason": "기록 호출 없음, 계산은 있어도 출력 없음"
    },
    {
      "id": "04A",
      "title": "조건의참갈래",
      "file": "예제/04A_조건의참갈래.ddn",
      "expected": [
        "8"
      ],
      "reason": "8>=5:왼쪽"
    },
    {
      "id": "04B",
      "title": "조건의거짓갈래",
      "file": "예제/04B_조건의거짓갈래.ddn",
      "expected": [
        "9"
      ],
      "reason": "3<9:오른쪽"
    },
    {
      "id": "04C",
      "title": "같을때",
      "file": "예제/04C_같을때.ddn",
      "expected": [
        "5"
      ],
      "reason": "5>=5:왼쪽, 값5"
    },
    {
      "id": "04D",
      "title": "음수비교",
      "file": "예제/04D_음수비교.ddn",
      "expected": [
        "-2"
      ],
      "reason": "-2>=-5:왼쪽"
    },
    {
      "id": "05A",
      "title": "정의안반복",
      "file": "예제/05A_정의안반복.ddn",
      "expected": [
        "15"
      ],
      "reason": "1+2+3+4+5=15"
    },
    {
      "id": "05B",
      "title": "반복범위바꾸기",
      "file": "예제/05B_반복범위바꾸기.ddn",
      "expected": [
        "6"
      ],
      "reason": "1+2+3=6"
    },
    {
      "id": "05C",
      "title": "처음부터거짓",
      "file": "예제/05C_처음부터거짓.ddn",
      "expected": [
        "0"
      ],
      "reason": "1<=0 거짓, 합계 초기값0"
    },
    {
      "id": "05D",
      "title": "한번반복",
      "file": "예제/05D_한번반복.ddn",
      "expected": [
        "1"
      ],
      "reason": "1 한 번 더함"
    },
    {
      "id": "06A",
      "title": "차림받는자리",
      "file": "예제/06A_차림받는자리.ddn",
      "expected": [
        "12"
      ],
      "reason": "2+4+6=12"
    },
    {
      "id": "06B",
      "title": "차림바꾸기",
      "file": "예제/06B_차림바꾸기.ddn",
      "expected": [
        "16"
      ],
      "reason": "1+3+5+7=16"
    },
    {
      "id": "06C",
      "title": "한원소차림",
      "file": "예제/06C_한원소차림.ddn",
      "expected": [
        "5"
      ],
      "reason": "원소 하나5를 한 번 더해5; 빈 차림 생성은 현재 본문 파서에서 미지원이므로 이 수업의 정상 사례로 약속하지 않는다"
    },
    {
      "id": "06D",
      "title": "순회와움직씨",
      "file": "예제/06D_순회와움직씨.ddn",
      "expected": [
        "2",
        "4",
        "6"
      ],
      "reason": "각 원소*2, 순서대로 기록"
    },
    {
      "id": "07A",
      "title": "요금규칙연결",
      "file": "예제/07A_요금규칙연결.ddn",
      "expected": [
        "2800",
        "2100"
      ],
      "reason": "1000+120*15=2800;2800*3/4=2100"
    },
    {
      "id": "07B",
      "title": "규칙값수정",
      "file": "예제/07B_규칙값수정.ddn",
      "expected": [
        "2500",
        "1875"
      ],
      "reason": "1000+100*15=2500;2500*3/4=1875"
    },
    {
      "id": "07C",
      "title": "할인경계",
      "file": "예제/07C_할인경계.ddn",
      "expected": [
        "2800"
      ],
      "reason": "할인율0:2800"
    },
    {
      "id": "07D",
      "title": "핀중복오류",
      "file": "예제/07D_핀중복오류.ddn",
      "expected": {
        "reject": "중복"
      },
      "reason": "같은 사용량 받는자리에 두 값을 주면 거부"
    },
    {
      "id": "08A",
      "title": "씨불일치",
      "file": "예제/08A_씨불일치.ddn",
      "expected": {
        "reject": "E_RUNTIME_TYPE_MISMATCH"
      },
      "reason": "수 핀에 글을 보내는 실수를 거부"
    },
    {
      "id": "08B",
      "title": "받는자리누락",
      "file": "예제/08B_받는자리누락.ddn",
      "expected": {
        "reject": "E_CALL_REQUIRED_PIN_MISSING"
      },
      "reason": "높이 누락 거부"
    },
    {
      "id": "08C",
      "title": "결과자리누락",
      "file": "예제/08C_결과자리누락.ddn",
      "expected": {
        "reject": "E_RESULTPLACE_UNASSIGNED"
      },
      "reason": "계산했지만 결과자리를 배정하지 않은 실수 거부"
    },
    {
      "id": "08D",
      "title": "조사모호",
      "file": "예제/08D_조사모호.ddn",
      "expected": {
        "reject": "모호"
      },
      "reason": "같은 조사로 두 핀 중 하나를 추측하지 않음"
    },
    {
      "id": "09A",
      "title": "용사와슬라임",
      "file": "예제/09A_용사와슬라임.ddn",
      "expected": [
        "7"
      ],
      "reason": "피해8-3=5;체력12-5=7"
    },
    {
      "id": "09B",
      "title": "방어규칙경계",
      "file": "예제/09B_방어규칙경계.ddn",
      "expected": [
        "0",
        "0"
      ],
      "reason": "방어력이 공격력 이상이면 피해0"
    },
    {
      "id": "09C",
      "title": "쓰러짐경계",
      "file": "예제/09C_쓰러짐경계.ddn",
      "expected": [
        "0"
      ],
      "reason": "체력보다 피해가 커도 남은체력0"
    },
    {
      "id": "09D",
      "title": "미로의열쇠",
      "file": "예제/09D_미로의열쇠.ddn",
      "expected": [
        "열림",
        "잠김"
      ],
      "reason": "열쇠가 있으면 열림, 없으면 잠김; 실시간 입력 게임 아님"
    },
    {
      "id": "10A",
      "title": "작은상점",
      "file": "예제/10A_작은상점.ddn",
      "expected": [
        "구매 가능",
        "2600"
      ],
      "reason": "800*3=2400;5000-2400=2600"
    },
    {
      "id": "10B",
      "title": "잔액부족",
      "file": "예제/10B_잔액부족.ddn",
      "expected": [
        "잔액 부족",
        "2000"
      ],
      "reason": "2000<2400, 구매하지 않는 판정"
    },
    {
      "id": "10C",
      "title": "잔액딱맞음",
      "file": "예제/10C_잔액딱맞음.ddn",
      "expected": [
        "구매 가능",
        "0"
      ],
      "reason": "2400>=2400;남은금액0"
    },
    {
      "id": "10D",
      "title": "규칙을여러번",
      "file": "예제/10D_규칙을여러번.ddn",
      "expected": [
        "잔액 부족",
        "2000",
        "구매 가능",
        "0",
        "구매 가능",
        "2600"
      ],
      "reason": "같은 셈씨/움직씨 규칙을 세 잔액에 적용; 잔액은 각 실험 입력이며 저장 게임 상태 아님"
    }
  ],
  "corrections": [
    {
      "id": "FIX07D",
      "title": "핀중복고치기",
      "expected": [
        "2800"
      ],
      "reason": "중복 인자만 제거;1000+120*15=2800",
      "file": "교정예제/FIX07D_핀중복고치기.ddn"
    },
    {
      "id": "FIX08A",
      "title": "씨맞추기",
      "expected": [
        "14"
      ],
      "reason": "글 대신 수7 전달;7*2=14",
      "file": "교정예제/FIX08A_씨맞추기.ddn"
    },
    {
      "id": "FIX08B",
      "title": "높이채우기",
      "expected": [
        "20"
      ],
      "reason": "높이5를 명시;8*5/2=20",
      "file": "교정예제/FIX08B_높이채우기.ddn"
    },
    {
      "id": "FIX08C",
      "title": "결과자리배정",
      "expected": [
        "14"
      ],
      "reason": "결과자리배정;7*2=14",
      "file": "교정예제/FIX08C_결과자리배정.ddn"
    },
    {
      "id": "FIX08D",
      "title": "이름핀으로명시",
      "expected": [
        "7"
      ],
      "reason": "조사추측대신 이름핀으로두받는자리명시;3+4=7",
      "file": "교정예제/FIX08D_이름핀으로명시.ddn"
    }
  ],
  "modifications": [
    {
      "id": "MOD01",
      "title": "개수수정",
      "from": "01C",
      "file": "수정실습/MOD01_개수수정.ddn",
      "expected": [
        "3200"
      ],
      "reason": "800*4=3200"
    },
    {
      "id": "MOD02",
      "title": "높이수정",
      "from": "02C",
      "file": "수정실습/MOD02_높이수정.ddn",
      "expected": [
        "24"
      ],
      "reason": "8*6/2=24"
    },
    {
      "id": "MOD03",
      "title": "출력호출제거",
      "from": "03C",
      "file": "수정실습/MOD03_출력호출제거.ddn",
      "expected": [],
      "reason": "기록 호출만 제거해 계산14는 있어도 출력 없음"
    },
    {
      "id": "MOD04",
      "title": "인자뒤바꾸기",
      "from": "04A",
      "file": "수정실습/MOD04_인자뒤바꾸기.ddn",
      "expected": [
        "8"
      ],
      "reason": "5<8:오른쪽8"
    },
    {
      "id": "MOD05",
      "title": "끝수수정",
      "from": "05A",
      "file": "수정실습/MOD05_끝수수정.ddn",
      "expected": [
        "6"
      ],
      "reason": "1+2+3=6"
    },
    {
      "id": "MOD06",
      "title": "차림수정",
      "from": "06D",
      "file": "수정실습/MOD06_차림수정.ddn",
      "expected": [
        "4",
        "8",
        "12"
      ],
      "reason": "차례로2*2,4*2,6*2"
    },
    {
      "id": "MOD07",
      "title": "사용량수정",
      "from": "07A",
      "file": "수정실습/MOD07_사용량수정.ddn",
      "expected": [
        "2500",
        "1875"
      ],
      "reason": "1000+100*15=2500;2500*3/4=1875"
    },
    {
      "id": "MOD08",
      "title": "교정후값수정",
      "from": "FIX08D",
      "file": "수정실습/MOD08_교정후값수정.ddn",
      "expected": [
        "11"
      ],
      "reason": "이름 핀5+6=11"
    },
    {
      "id": "MOD09",
      "title": "방어력수정",
      "from": "09A",
      "file": "수정실습/MOD09_방어력수정.ddn",
      "expected": [
        "12"
      ],
      "reason": "공격=방어:피해0;체력12"
    },
    {
      "id": "MOD10",
      "title": "상점잔액수정",
      "from": "10A",
      "file": "수정실습/MOD10_상점잔액수정.ddn",
      "expected": [
        "잔액 부족",
        "2000"
      ],
      "reason": "상품금액2400>잔액2000:구매거부,잔액2000"
    }
  ]
};
export const FIRST_STEPS_PROJECTS = [
  ...FIRST_STEPS_COURSE_MANIFEST.cases,
  ...FIRST_STEPS_COURSE_MANIFEST.corrections,
  ...FIRST_STEPS_COURSE_MANIFEST.modifications
].map((row) => ({
  id: row.project_id || `first_steps_${row.id}`,
  nativeStandaloneSource: row.nativeStandaloneSource === true,
  title: row.project_title || `첫걸음 ${row.id} · ${row.title}`,
  kind: "정의 중심 첫걸음",
  summary: "설명 읽기 → 예상하기 → 실행하기 → 수정하기 → 해설 확인",
  tags: ["첫걸음", row.id, "셈씨", "움직씨"],
  primaryView: "console",
  viewGrade: "output-only",
  viewPolicy: {primary: "실행 출력", side: "", overlay: "", magnifier: ""},
  lessonDisposition: Array.isArray(row.expected) ? "예상하고 실행하기" : "오류 읽기 · 의도한 거부",
  files: [{path: row.source_path || "lesson.ddn", role: "main", url: `../../first_steps/${row.file}`}],
  course: row
}));

// 위임된 교육 데이터: 실제 수학과 보임 판정은 공통 제품이 담당한다.
const V2_SAVED_PROOF_LESSONS = [
  {
    "id": "v2_saved_proof_algebra",
    "title": "같음 원문 저장과 재검사",
    "kind": "V2-03 원문 보관 수업",
    "summary": "원문 저장·재열기 뒤 명시적으로 보임을 다시 검사합니다.",
    "tags": [
      "V2",
      "보임",
      "저장",
      "재검사"
    ],
    "primaryView": "console",
    "viewGrade": "output-only",
    "nativeAuthoredFiles": [
      {
        "path": "root.ddn",
        "role": "root",
        "url": "../../first_steps_rc1/v2_examples_r1/algebra/root.ddn"
      },
      {
        "path": "app.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/algebra/app.ddn"
      },
      {
        "path": "lib/proof.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/algebra/lib/proof.ddn"
      }
    ],
    "files": [
      {
        "path": "root.ddn",
        "role": "root",
        "url": "../../first_steps_rc1/v2_examples_r1/algebra/root.ddn"
      },
      {
        "path": "app.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/algebra/app.ddn"
      },
      {
        "path": "lib/proof.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/algebra/lib/proof.ddn"
      }
    ],
    "portableSnapshot": {
      "source_profile_identity": "v1-core-v25",
      "local_package_payload": {
        "kind": "seamgrim.local_project_tree.v0",
        "root_path": "root.ddn",
        "entry_path": "app.ddn",
        "files": [
          {
            "path": "root.ddn",
            "role": "root"
          },
          {
            "path": "app.ddn",
            "role": "ddn_source"
          },
          {
            "path": "lib/proof.ddn",
            "role": "ddn_source"
          }
        ]
      }
    },
    "sourceProfileIdentity": "v1-core-v25",
    "course": {
      "id": "V2-03-algebra",
      "title": "같음 원문 저장과 재검사",
      "asset_root": "../../first_steps_rc1/v2_examples_r1/",
      "student_doc": "학생용_보임저장.html",
      "teacher_doc": "교사용.html#proof",
      "reason": "원문 저장과 검사 결과의 현재 유효성은 다릅니다.",
      "expected_description": "lib/proof.ddn 선택·명시 보임 검사: 조건부 정의1개 수용. 재열기 전후 자동 실행/검사 없음. app의 출력5는 보임 판정이 아님."
    }
  },
  {
    "id": "v2_saved_proof_structure",
    "title": "구조 법칙 원문 저장과 재검사",
    "kind": "V2-03 원문 보관 수업",
    "summary": "원문 저장·재열기 뒤 명시적으로 보임을 다시 검사합니다.",
    "tags": [
      "V2",
      "보임",
      "저장",
      "재검사"
    ],
    "primaryView": "console",
    "viewGrade": "output-only",
    "nativeAuthoredFiles": [
      {
        "path": "root.ddn",
        "role": "root",
        "url": "../../first_steps_rc1/v2_examples_r1/structure/root.ddn"
      },
      {
        "path": "app.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/structure/app.ddn"
      },
      {
        "path": "lib/proof.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/structure/lib/proof.ddn"
      }
    ],
    "files": [
      {
        "path": "root.ddn",
        "role": "root",
        "url": "../../first_steps_rc1/v2_examples_r1/structure/root.ddn"
      },
      {
        "path": "app.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/structure/app.ddn"
      },
      {
        "path": "lib/proof.ddn",
        "role": "ddn_source",
        "url": "../../first_steps_rc1/v2_examples_r1/structure/lib/proof.ddn"
      }
    ],
    "portableSnapshot": {
      "source_profile_identity": "v1-core-v25",
      "local_package_payload": {
        "kind": "seamgrim.local_project_tree.v0",
        "root_path": "root.ddn",
        "entry_path": "app.ddn",
        "files": [
          {
            "path": "root.ddn",
            "role": "root"
          },
          {
            "path": "app.ddn",
            "role": "ddn_source"
          },
          {
            "path": "lib/proof.ddn",
            "role": "ddn_source"
          }
        ]
      }
    },
    "sourceProfileIdentity": "v1-core-v25",
    "course": {
      "id": "V2-03-structure",
      "title": "구조 법칙 원문 저장과 재검사",
      "asset_root": "../../first_steps_rc1/v2_examples_r1/",
      "student_doc": "학생용_보임저장.html",
      "teacher_doc": "교사용.html#proof",
      "reason": "원문 저장과 검사 결과의 현재 유효성은 다릅니다.",
      "expected_description": "lib/proof.ddn 선택·명시 보임 검사: 조건부 정의2개 수용. 재열기 전후 자동 실행/검사 없음. app의 출력5는 보임 판정이 아님."
    }
  }
];
FIRST_STEPS_PROJECTS.push(...V2_SAVED_PROOF_LESSONS);

// 교재 고정 반환: 계산/경계 읽기와 같은 원문 저장 실험.
FIRST_STEPS_PROJECTS.push({
  "id": "v2_piecewise_tariff_course",
  "tags": ["V2", "조건별", "요금", "저장"],
  "title": "조건별 요금 · 저장하고 이어가기",
  "kind": "V2-05 첫걸음",
  "summary": "요금의 입력과 마디 횟수를 구분하고 저장한 세계를 이어 실행합니다.",
  "primaryView": "console",
  "viewGrade": "output-only",
  "sourceProfileIdentity": "v1-core-v25",
  "nativeAuthoredFiles": [
    {
      "path": "root.ddn",
      "role": "root",
      "url": "../../first_steps_rc1/v2_piecewise_r1/저장실험/root.ddn"
    },
    {
      "path": "app.ddn",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/v2_piecewise_r1/저장실험/app.ddn"
    }
  ],
  "files": [
    {
      "path": "root.ddn",
      "role": "root",
      "url": "../../first_steps_rc1/v2_piecewise_r1/저장실험/root.ddn"
    },
    {
      "path": "app.ddn",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/v2_piecewise_r1/저장실험/app.ddn"
    }
  ],
  "course": {
    "asset_root": "../../first_steps_rc1/v2_piecewise_r1/",
    "student_doc": "index.html",
    "teacher_doc": "교사용.html#store",
    "expected_description": "사용량3의 요금30/1. 저장2마디+새프로세스 다음2=연속4마디. 마지막 횟수4는 사용량이 아님.",
    "id": "V2-05",
    "title": "조건별 요금과 경계 · 저장하고 이어가기",
    "reason": "계산 입력3의 요금30/1과 실행 마디 횟수를 구분합니다. 입력4의 CLI 계산은50/1입니다."
  },
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "root.ddn",
      "entry_path": "app.ddn",
      "files": [
        {
          "path": "root.ddn",
          "role": "root"
        },
        {
          "path": "app.ddn",
          "role": "ddn_source"
        }
      ]
    }
  }
});

// Fixed CLI lessons: opening documentation does not switch/run the current project.
FIRST_STEPS_PROJECTS.push(...[
  {
    "id": "v2_structure_identity_7",
    "title": "항등 짜임 · 정의와 근거 읽기",
    "kind": "V2-04 CLI 수업",
    "summary": "교재와 원문을 내려받아 CLI에서 실행합니다. 브라우저 실행·Native 저장 수업이 아닙니다.",
    "tags": [
      "V2",
      "짜임",
      "CLI 수업"
    ],
    "documentOnly": true,
    "lessonDisposition": "CLI 교재 열기 · 브라우저 실행 아님",
    "files": [],
    "course": {
      "id": "V2-04",
      "asset_root": "../../first_steps_rc1/v2_structure_identity_r1/",
      "student_doc": "학생용.html#definition",
      "teacher_doc": "교사용.html"
    }
  },
  {
    "id": "v2_structure_identity_other",
    "title": "항등 짜임 · 상수7과 항등 구별하기",
    "kind": "V2-04 CLI 수업",
    "summary": "교재와 원문을 내려받아 CLI에서 실행합니다. 브라우저 실행·Native 저장 수업이 아닙니다.",
    "tags": [
      "V2",
      "짜임",
      "CLI 수업"
    ],
    "documentOnly": true,
    "lessonDisposition": "CLI 교재 열기 · 브라우저 실행 아님",
    "files": [],
    "course": {
      "id": "V2-04",
      "asset_root": "../../first_steps_rc1/v2_structure_identity_r1/",
      "student_doc": "학생용.html#counterexample",
      "teacher_doc": "교사용.html"
    }
  }
]);

// Fixed module/API follow-up documentation; no project switch or execution.
FIRST_STEPS_PROJECTS.push({
  "id": "v2_structure_identity_module_native",
  "title": "항등 짜임 · 모듈과 원문 보관",
  "kind": "V2-04 CLI/API 수업",
  "summary": "실제 시작 정의와 C: 한정 Native API 원문 저장·재열기를 배웁니다. 브라우저 실행·세계 이어 실행 수업은 아닙니다.",
  "tags": [
    "V2",
    "항등",
    "CLI/API",
    "원문 저장"
  ],
  "documentOnly": true,
  "lessonDisposition": "CLI/API 교재 열기 · 브라우저 실행 아님",
  "files": [],
  "course": {
    "id": "V2-04",
    "asset_root": "../../first_steps_rc1/v2_structure_identity_r1/",
    "student_doc": "모듈과저장.html",
    "teacher_doc": "모듈과저장_교사용.html"
  }
});


// Display-language course; original lesson and explicit supplied root/entry preserved.
FIRST_STEPS_PROJECTS.push({
  "id": "v2_display_saved_source",
  "title": "안내 언어와 저장 상태",
  "kind": "V2-06 표시 언어 수업",
  "primaryView": "console",
  "viewGrade": "output-only",
  "summary": "안내만 바꾸고 원문·계산·저장 상태는 보존합니다.",
  "sourceProfileIdentity": "v1-core-v25",
  "files": [
    {
      "path": "뿌리.ddn",
      "role": "root",
      "url": "../../first_steps_rc1/v2_display_r1/저장안내/뿌리.ddn"
    },
    {
      "path": "app/main.ddn",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/v2_display_r1/저장안내/app/main.ddn"
    }
  ],
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.ddn",
      "role": "root",
      "url": "../../first_steps_rc1/v2_display_r1/저장안내/뿌리.ddn"
    },
    {
      "path": "app/main.ddn",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/v2_display_r1/저장안내/app/main.ddn"
    }
  ],
  "course": {
    "asset_root": "../../first_steps_rc1/v2_display_r1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected_description": "실행 출력0. 표시 전환만으로 원문·세계·실행·저장 종류 불변, 저장 실패는 성공으로 바뀌지 않음.",
    "id": "V2-06",
    "title": "같은 한국어 원문 · 한국어와 일본어 안내",
    "reason": "표시 언어와 원문·계산·저장 상태를 구분합니다."
  },
  "tags": [
    "V2",
    "표시 언어",
    "학습"
  ],
  "viewPolicy": {
    "primary": "실행 출력",
    "side": "",
    "overlay": "",
    "magnifier": ""
  },
  "lessonDisposition": "같은 한국어 원문으로 안내 비교",
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.ddn",
      "entry_path": "app/main.ddn",
      "files": [
        {
          "path": "뿌리.ddn",
          "role": "root"
        },
        {
          "path": "app/main.ddn",
          "role": "ddn_source"
        }
      ]
    }
  }
});

FIRST_STEPS_PROJECTS.push({
  id: 'v3_uniform_motion_notebook', title: '등속운동 · 속도와 도달 시간', kind: 'V3 탐구 노트',
  summary: '같은 시각의 위치를 비교하고 목표까지 걸리는 시간을 검산합니다.',
  tags: ['V3', '수학', '물리', '탐구 노트'], primaryView: 'console', viewGrade: 'output-only',
  sourceProfileIdentity: 'v1-core-v25',
  files: [
    { path: 'root.ddn', role: 'root', url: '../../first_steps_v3/uniform_motion/root.ddn' },
    { path: 'main.ddn', role: 'ddn_source', url: '../../first_steps_v3/uniform_motion/main.ddn' },
  ],
  nativeAuthoredFiles: [
    { path: 'root.ddn', role: 'root', url: '../../first_steps_v3/uniform_motion/root.ddn' },
    { path: 'main.ddn', role: 'ddn_source', url: '../../first_steps_v3/uniform_motion/main.ddn' },
  ],
  course: { id: 'V3-M1-M7', title: '속도가 달라지면 언제 도착할까?',
    asset_root: '../../first_steps_v3/uniform_motion/', student_doc: 'index.html', teacher_doc: '교사용.md',
    expected_description: '위치1/4/7m와 속도 변경1/5/9m를 비교합니다. 응용 실험에서는 출발점·속도·목표를 직접 정하고, 속도0의 해 없음·비유일을 구분합니다.' },
  portableSnapshot: { source_profile_identity: 'v1-core-v25', local_package_payload: {
    kind: 'seamgrim.local_project_tree.v0', root_path: 'root.ddn', entry_path: 'main.ddn',
    files: [{ path: 'root.ddn', role: 'root' }, { path: 'main.ddn', role: 'ddn_source' }],
  } },
});

FIRST_STEPS_PROJECTS.push({
  id: 'v3_water_tank_notebook', title: '물탱크 · 유량과 도달 시간', kind: 'V3 탐구 노트',
  summary: '유량 하나를 바꿔 물의 양과 목표 도달 시간을 비교합니다.',
  tags: ['V3', '수학', '과학', '탐구 노트'], primaryView: 'console', viewGrade: 'output-only',
  sourceProfileIdentity: 'v1-core-v25',
  files: [
    { path: 'root.ddn', role: 'root', url: '../../first_steps_v3/uniform_motion/water_tank/root.ddn' },
    { path: 'main.ddn', role: 'ddn_source', url: '../../first_steps_v3/uniform_motion/water_tank/main.ddn' },
  ],
  nativeAuthoredFiles: [
    { path: 'root.ddn', role: 'root', url: '../../first_steps_v3/uniform_motion/water_tank/root.ddn' },
    { path: 'main.ddn', role: 'ddn_source', url: '../../first_steps_v3/uniform_motion/water_tank/main.ddn' },
  ],
  course: { id: 'V3-WATER', title: '유량이 달라지면 언제 채워질까?',
    asset_root: '../../first_steps_v3/uniform_motion/water_tank/', student_doc: 'index.html', teacher_doc: '교사용.md',
    expected_description: 'A의 양1/500·1/200·1/125m³와 목표120초, 유량만 바꾼 B의 양1/500·11/1000·1/50m³와 목표40초를 비교합니다.' },
  portableSnapshot: { source_profile_identity: 'v1-core-v25', local_package_payload: {
    kind: 'seamgrim.local_project_tree.v0', root_path: 'root.ddn', entry_path: 'main.ddn',
    files: [{ path: 'root.ddn', role: 'root' }, { path: 'main.ddn', role: 'ddn_source' }],
  } },
});

// 교재 통합 후보: dev exact lease 뒤 첫 정상 사용자 흐름만 활성화.
FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_첫계산_1",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 첫 계산",
  "kind": "또니랑 입문",
  "summary": "계산을 읽고 수를 고친 뒤 실제 출력과 관찰을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "계산",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "첫계산.또니",
      "role": "main",
      "url": "../../first_steps_rc1/첫계산_기초01/첫계산.또니"
    }
  ],
  "course": {
    "id": "한국어입문_첫계산_1",
    "title": "첫 계산",
    "lesson": 1,
    "asset_root": "../../first_steps_rc1/첫계산_기초01/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "5"
    ],
    "expected_description": "합의2를7로 바꾸면10입니다. 명시 실행 뒤 실제 출력과 결과 관찰을 대조합니다.",
    "reason": "원문 편집·명시 실행·파일 보관과 세계 이어하기를 구별합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_값과글_2",
  "title": "한국어 입문 · 값과 글",
  "kind": "또니랑 입문",
  "summary": "수와 글을 구별하고 이름으로 계산을 이어갑니다.",
  "tags": [
    "또니랑",
    "입문",
    "수",
    "글",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "값과글.또니",
      "role": "main",
      "url": "../../first_steps_rc1/값과글_기초02/값과글.또니"
    }
  ],
  "course": {
    "id": "한국어입문_값과글_2",
    "title": "값과 글",
    "lesson": 2,
    "asset_root": "../../first_steps_rc1/값과글_기초02/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "계산 결과",
      "8"
    ],
    "expected_description": "기준값의4를3으로 바꾸면 설명 글은 그대로이고 결과는6입니다.",
    "reason": "수·글·관찰을 같은 원문에서 읽고 수정해 명시 실행합니다. 파일 보관과 세계 이어하기는 구분합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_조건갈라지기_3",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 조건에 따라 갈라지기",
  "kind": "또니랑 입문",
  "summary": "조건에 따라 갈라지기 원문을 읽고 한 부분을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "조건갈라지기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/조건갈라지기_기초03/조건갈라지기.또니"
    }
  ],
  "course": {
    "id": "한국어입문_조건갈라지기_3",
    "title": "조건에 따라 갈라지기",
    "lesson": 3,
    "asset_root": "../../first_steps_rc1/조건갈라지기_기초03/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "통과"
    ],
    "expected_description": "기본 통과 → 지정한 부분을 수정하면 다시 해 보기.",
    "reason": "기존 원문을 같은 작업대에서 열기·편집·명시 실행합니다. 원문 보관과 세계 이어하기는 구분합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_반복합계_4",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 반복과 합계",
  "kind": "또니랑 입문",
  "summary": "반복과 합계 원문을 읽고 한 부분을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "반복합계.또니",
      "role": "main",
      "url": "../../first_steps_rc1/반복합계_기초04/반복합계.또니"
    }
  ],
  "course": {
    "id": "한국어입문_반복합계_4",
    "title": "반복과 합계",
    "lesson": 4,
    "asset_root": "../../first_steps_rc1/반복합계_기초04/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "15"
    ],
    "expected_description": "기본 15 → 지정한 부분을 수정하면 6.",
    "reason": "기존 원문을 같은 작업대에서 열기·편집·명시 실행합니다. 원문 보관과 세계 이어하기는 구분합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_넓이구하기_5",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 조사로 넓이 구하기",
  "kind": "또니랑 입문",
  "summary": "조사로 넓이 구하기 원문을 읽고 한 부분을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "넓이구하기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/넓이구하기_기초05/넓이구하기.또니"
    }
  ],
  "course": {
    "id": "한국어입문_넓이구하기_5",
    "title": "조사로 넓이 구하기",
    "lesson": 5,
    "asset_root": "../../first_steps_rc1/넓이구하기_기초05/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "12"
    ],
    "expected_description": "기본 12 → 지정한 부분을 수정하면 24.",
    "reason": "기존 원문을 같은 작업대에서 열기·편집·명시 실행합니다. 원문 보관과 세계 이어하기는 구분합니다. 선언한 조사로 호출하며 미등록 수치 조사는 거부합니다. 구판 저장 원문은 그대로 열어 교정합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_참거짓비교_6",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 참거짓과 비교",
  "kind": "또니랑 입문",
  "summary": "참거짓과 비교 원문을 읽고 한 부분을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "참거짓비교.또니",
      "role": "main",
      "url": "../../first_steps_rc1/참거짓비교_기초06/참거짓비교.또니"
    }
  ],
  "course": {
    "id": "한국어입문_참거짓비교_6",
    "title": "참거짓과 비교",
    "lesson": 6,
    "asset_root": "../../first_steps_rc1/참거짓비교_기초06/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "참",
      "거짓",
      "거짓"
    ],
    "expected_description": "기본 참, 거짓, 거짓 → 지정한 부분을 수정하면 거짓, 거짓, 거짓.",
    "reason": "기존 원문을 같은 작업대에서 열기·편집·명시 실행합니다. 원문 보관과 세계 이어하기는 구분합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_가격다시놓기_7",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 값을 다시 놓기",
  "kind": "또니랑 입문",
  "summary": "값을 다시 놓기 원문을 읽고 한 부분을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "가격다시놓기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/가격다시놓기_기초07/가격다시놓기.또니"
    }
  ],
  "course": {
    "id": "한국어입문_가격다시놓기_7",
    "title": "값을 다시 놓기",
    "lesson": 7,
    "asset_root": "../../first_steps_rc1/가격다시놓기_기초07/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "780"
    ],
    "expected_description": "기본 780 → 지정한 부분을 수정하면 980.",
    "reason": "기존 원문을 같은 작업대에서 열기·편집·명시 실행합니다. 원문 보관과 세계 이어하기는 구분합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_글과주석_8",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 글과 주석",
  "kind": "또니랑 입문",
  "summary": "글과 주석 원문을 읽고 한 부분을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "글과주석.또니",
      "role": "main",
      "url": "../../first_steps_rc1/글과주석_기초08/글과주석.또니"
    }
  ],
  "course": {
    "id": "한국어입문_글과주석_8",
    "title": "글과 주석",
    "lesson": 8,
    "asset_root": "../../first_steps_rc1/글과주석_기초08/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "반가워요 🌱",
      "007"
    ],
    "expected_description": "기본 반가워요 🌱, 007 → 지정한 부분을 수정하면 반가워요 🌱, 008.",
    "reason": "기존 원문을 같은 작업대에서 열기·편집·명시 실행합니다. 원문 보관과 세계 이어하기는 구분합니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_차림과옛값_9",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 차림과 옛값",
  "kind": "또니랑 입문",
  "summary": "차림과 옛값 원문 한 곳을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "차림과옛값.또니",
      "role": "main",
      "url": "../../first_steps_rc1/차림과옛값_기초09/차림과옛값.또니"
    }
  ],
  "course": {
    "id": "한국어입문_차림과옛값_9",
    "title": "차림과 옛값",
    "lesson": 9,
    "asset_root": "../../first_steps_rc1/차림과옛값_기초09/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "6",
      "15"
    ],
    "expected_description": "기본 6, 15 → 지정 수정 후 6, 18.",
    "reason": "기존 R24 원문을 같은 작업대에서 읽고 편집·명시 실행합니다. 현재 고정 Worker의 기본·수정 실행과 Native 원문 ACK·새 프로세스 재열기·명시 재실행을 확인했습니다. 결과·세계는 보관하지 않으며 실제 브라우저 화면은 미검증입니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_서로다른값묶기_10",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 서로 다른 값을 묶기",
  "kind": "또니랑 입문",
  "summary": "서로 다른 값을 묶기 원문 한 곳을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "서로다른값묶기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/서로다른값묶기_기초10/서로다른값묶기.또니"
    }
  ],
  "course": {
    "id": "한국어입문_서로다른값묶기_10",
    "title": "서로 다른 값을 묶기",
    "lesson": 10,
    "asset_root": "../../first_steps_rc1/서로다른값묶기_기초10/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "12",
      "달"
    ],
    "expected_description": "기본 12, 달 → 지정 수정 후 18, 달.",
    "reason": "기존 R24 원문을 같은 작업대에서 읽고 편집·명시 실행합니다. 현재 고정 Worker의 기본·수정 실행과 Native 원문 ACK·새 프로세스 재열기·명시 재실행을 확인했습니다. 결과·세계는 보관하지 않으며 실제 브라우저 화면은 미검증입니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_텐서의행과열_11",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 텐서의 행과 열",
  "kind": "또니랑 입문",
  "summary": "텐서의 행과 열 원문 한 곳을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "텐서의행과열.또니",
      "role": "main",
      "url": "../../first_steps_rc1/텐서의행과열_기초11/텐서의행과열.또니"
    }
  ],
  "course": {
    "id": "한국어입문_텐서의행과열_11",
    "title": "텐서의 행과 열",
    "lesson": 11,
    "asset_root": "../../first_steps_rc1/텐서의행과열_기초11/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "2",
      "7"
    ],
    "expected_description": "기본 2, 7 → 지정 수정 후 2, 8.",
    "reason": "기존 R24 원문을 같은 작업대에서 읽고 편집·명시 실행합니다. 현재 고정 Worker의 기본·수정 실행과 Native 원문 ACK·새 프로세스 재열기·명시 재실행을 확인했습니다. 결과·세계는 보관하지 않으며 실제 브라우저 화면은 미검증입니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_길이단위바꾸기_12",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 길이 단위 바꾸기",
  "kind": "또니랑 입문",
  "summary": "길이 단위 바꾸기 원문 한 곳을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "길이단위바꾸기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/길이단위바꾸기_기초12/길이단위바꾸기.또니"
    }
  ],
  "course": {
    "id": "한국어입문_길이단위바꾸기_12",
    "title": "길이 단위 바꾸기",
    "lesson": 12,
    "asset_root": "../../first_steps_rc1/길이단위바꾸기_기초12/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "1.5@m"
    ],
    "expected_description": "기본 1.5@m → 지정 수정 후 2.5@m.",
    "reason": "기존 R24 원문을 같은 작업대에서 읽고 편집·명시 실행합니다. 현재 고정 Worker의 기본·수정 실행과 Native 원문 ACK·새 프로세스 재열기·명시 재실행을 확인했습니다. 결과·세계는 보관하지 않으며 실제 브라우저 화면은 미검증입니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_정확한분수계산_13",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 정확한 분수 계산",
  "kind": "또니랑 입문",
  "summary": "정확한 분수 계산 원문 한 곳을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "정확한분수계산.또니",
      "role": "main",
      "url": "../../first_steps_rc1/정확한분수계산_기초13/정확한분수계산.또니"
    }
  ],
  "course": {
    "id": "한국어입문_정확한분수계산_13",
    "title": "정확한 분수 계산",
    "lesson": 13,
    "asset_root": "../../first_steps_rc1/정확한분수계산_기초13/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "1/2",
      "1/1"
    ],
    "expected_description": "기본 1/2, 1/1 → 지정 수정 후 1/2, 2/1.",
    "reason": "기존 R24 원문을 같은 작업대에서 읽고 편집·명시 실행합니다. 현재 고정 Worker의 기본·수정 실행과 Native 원문 ACK·새 프로세스 재열기·명시 재실행을 확인했습니다. 결과·세계는 보관하지 않으며 실제 브라우저 화면은 미검증입니다."
  }
});

FIRST_STEPS_PROJECTS.push({
  "id": "한국어입문_조사역할_14",
  "nativeStandaloneSource": true,
  "title": "한국어 입문 · 조사의 역할",
  "kind": "또니랑 입문",
  "summary": "조사의 역할 원문 한 곳을 고쳐 실제 출력을 비교합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "조사로두배구하기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/조사역할_기초14/조사로두배구하기.또니"
    }
  ],
  "course": {
    "id": "한국어입문_조사역할_14",
    "title": "조사의 역할",
    "lesson": 14,
    "asset_root": "../../first_steps_rc1/조사역할_기초14/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "12"
    ],
    "expected_description": "기본 12 → 지정 수정 후 16.",
    "reason": "기존 R24 원문을 같은 작업대에서 읽고 편집·명시 실행합니다. 현재 고정 Worker의 기본·수정 실행과 Native 원문 ACK·새 프로세스 재열기·명시 재실행을 확인했습니다. 결과·세계는 보관하지 않으며 실제 브라우저 화면은 미검증입니다. 과거 미등록 조사 수용 실패는 보존하며 이번 연결을 모든 조사 오류의 해결로 표시하지 않습니다."
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "움직씨결과_기초15",
  "title": "한국어 입문 · 결과가 있는 움직씨",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "점수올리기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/움직씨결과_기초15/점수올리기.또니"
    }
  ],
  "course": {
    "id": "움직씨결과_기초15",
    "title": "결과가 있는 움직씨",
    "lesson": 15,
    "asset_root": "../../first_steps_rc1/움직씨결과_기초15/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "8"
    ],
    "expected_description": "기본 8 → 지정 수정 후 11.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeStandaloneSource": true
});


FIRST_STEPS_PROJECTS.push({
  "id": "선택자리_기초16",
  "title": "한국어 입문 · 결과를 도착 자리에 놓기",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "선택자리에놓기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/선택자리_기초16/선택자리에놓기.또니"
    }
  ],
  "course": {
    "id": "선택자리_기초16",
    "title": "결과를 도착 자리에 놓기",
    "lesson": 16,
    "asset_root": "../../first_steps_rc1/선택자리_기초16/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "7"
    ],
    "expected_description": "기본 7 → 지정 수정 후 9.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeStandaloneSource": true
});


FIRST_STEPS_PROJECTS.push({
  "id": "구매결과_기초17",
  "title": "한국어 입문 · 성공과 실패를 값으로 다루기",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "구매결과살피기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/구매결과_기초17/구매결과살피기.또니"
    }
  ],
  "course": {
    "id": "구매결과_기초17",
    "title": "성공과 실패를 값으로 다루기",
    "lesson": 17,
    "asset_root": "../../first_steps_rc1/구매결과_기초17/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "재고 없음"
    ],
    "expected_description": "기본 재고 없음 → 지정 수정 후 3.",
    "reason": "원문·명목 등록자료를 Native에 보관하고 같은 정체로 재열어 명시 실행합니다. 결과·세계 진행·과거 출력은 보관하지 않습니다. 브라우저 클릭·독립판정·Human 확인은 별도입니다."
  },
  "nativeStandaloneSource": true
});


FIRST_STEPS_PROJECTS.push({
  "id": "여러파일_기초18",
  "title": "한국어 입문 · 여러 파일 작품",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/여러파일_기초18/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/여러파일_기초18/앱.또니"
    },
    {
      "path": "계산.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/여러파일_기초18/계산.또니"
    }
  ],
  "course": {
    "id": "여러파일_기초18",
    "title": "여러 파일 작품",
    "lesson": 18,
    "asset_root": "../../first_steps_rc1/여러파일_기초18/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "20"
    ],
    "expected_description": "기본 20 → 지정 수정 후 24.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/여러파일_기초18/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/여러파일_기초18/앱.또니"
    },
    {
      "path": "계산.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/여러파일_기초18/계산.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        },
        {
          "path": "계산.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "이름범위_기초19",
  "title": "한국어 입문 · 이름의 범위와 가림",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "이름의범위살피기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/이름범위_기초19/이름의범위살피기.또니"
    }
  ],
  "course": {
    "id": "이름범위_기초19",
    "title": "이름의 범위와 가림",
    "lesson": 19,
    "asset_root": "../../first_steps_rc1/이름범위_기초19/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "4",
      "9"
    ],
    "expected_description": "기본 4, 9 → 지정 수정 후 8, 9.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeStandaloneSource": true
});


FIRST_STEPS_PROJECTS.push({
  "id": "점수표필드_기초20",
  "title": "한국어 입문 · 이름 붙은 필드와 묶음",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "점수표필드읽기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/점수표필드_기초20/점수표필드읽기.또니"
    }
  ],
  "course": {
    "id": "점수표필드_기초20",
    "title": "이름 붙은 필드와 묶음",
    "lesson": 20,
    "asset_root": "../../first_steps_rc1/점수표필드_기초20/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "연두",
      "8"
    ],
    "expected_description": "기본 연두, 8 → 지정 수정 후 연두, 9.",
    "reason": "원문·명목 등록자료를 Native에 보관하고 같은 정체로 재열어 명시 실행합니다. 결과·세계 진행·과거 출력은 보관하지 않습니다. 브라우저 클릭·독립판정·Human 확인은 별도입니다."
  },
  "nativeStandaloneSource": true
});


FIRST_STEPS_PROJECTS.push({
  "id": "대상가리킴_기초21",
  "title": "한국어 입문 · 대상과 가리킴",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "대상가리킴비교.또니",
      "role": "main",
      "url": "../../first_steps_rc1/대상가리킴_기초21/대상가리킴비교.또니"
    }
  ],
  "course": {
    "id": "대상가리킴_기초21",
    "title": "대상과 가리킴",
    "lesson": 21,
    "asset_root": "../../first_steps_rc1/대상가리킴_기초21/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "참"
    ],
    "expected_description": "기본 참 → 지정 수정 후 참.",
    "reason": "이 원문의 Native 보관은 현재 지원되지 않습니다. 실행 지원과 별개이며, 저장 거부는 원문을 잃지 않고 표시됩니다. 원문 파일 저장을 선택할 수 있지만 Native 내구 저장 성공으로 대신하지 않습니다. 파일 내려받기·새 프로세스 파일 재열기의 실제 브라우저 확인은 아직 미실행입니다. 실제 브라우저 확인은 별도입니다."
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "관찰값_기초22",
  "title": "한국어 입문 · 관찰을 원문에 적기",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "관찰값발행하기.또니",
      "role": "main",
      "url": "../../first_steps_rc1/관찰값_기초22/관찰값발행하기.또니"
    }
  ],
  "course": {
    "id": "관찰값_기초22",
    "title": "관찰을 원문에 적기",
    "lesson": 22,
    "asset_root": "../../first_steps_rc1/관찰값_기초22/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "21"
    ],
    "expected_description": "기본 21 → 지정 수정 후 24.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeStandaloneSource": true
});


FIRST_STEPS_PROJECTS.push({
  "id": "원그림_기초23",
  "title": "한국어 입문 · 원문으로 장면 그리기",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "space2d",
  "viewGrade": "explicit",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/원그림_기초23/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/원그림_기초23/앱.또니"
    }
  ],
  "course": {
    "id": "원그림_기초23",
    "title": "원문으로 장면 그리기",
    "lesson": 23,
    "asset_root": "../../first_steps_rc1/원그림_기초23/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "1.25"
    ],
    "expected_description": "기본 1.25 → 지정 수정 후 0.75.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/원그림_기초23/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/원그림_기초23/앱.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "으뜸겹곁_기초24",
  "title": "한국어 입문 · 으뜸·겹·곁의 역할",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "space2d",
  "viewGrade": "explicit",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/으뜸겹곁_기초24/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/으뜸겹곁_기초24/앱.또니"
    }
  ],
  "course": {
    "id": "으뜸겹곁_기초24",
    "title": "으뜸·겹·곁의 역할",
    "lesson": 24,
    "asset_root": "../../first_steps_rc1/으뜸겹곁_기초24/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "0.5"
    ],
    "expected_description": "기본 0.5 → 지정 수정 후 0.5.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/으뜸겹곁_기초24/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/으뜸겹곁_기초24/앱.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "대상돋_기초25",
  "title": "한국어 입문 · 대상을 골라 돋으로 보기",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "space2d",
  "viewGrade": "explicit",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/대상돋_기초25/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/대상돋_기초25/앱.또니"
    }
  ],
  "course": {
    "id": "대상돋_기초25",
    "title": "대상을 골라 돋으로 보기",
    "lesson": 25,
    "asset_root": "../../first_steps_rc1/대상돋_기초25/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "1"
    ],
    "expected_description": "기본 1 → 지정 수정 후 1.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/대상돋_기초25/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/대상돋_기초25/앱.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "마디상태_기초26",
  "title": "한국어 입문 · 마디와 상태",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/마디상태_기초26/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/마디상태_기초26/앱.또니"
    }
  ],
  "course": {
    "id": "마디상태_기초26",
    "title": "마디와 상태",
    "lesson": 26,
    "asset_root": "../../first_steps_rc1/마디상태_기초26/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "1",
      "2",
      "3"
    ],
    "expected_description": "기본 1, 2, 3 → 지정 수정 후 11, 12, 13.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/마디상태_기초26/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/마디상태_기초26/앱.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "난수범위_기초27",
  "title": "한국어 입문 · 난수와 재현 조건",
  "kind": "또니랑 입문",
  "summary": "기존 원문을 같은 작업대에서 읽고 직접 수정·명시 실행합니다.",
  "tags": [
    "또니랑",
    "입문",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/난수범위_기초27/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/난수범위_기초27/앱.또니"
    }
  ],
  "course": {
    "id": "난수범위_기초27",
    "title": "난수와 재현 조건",
    "lesson": 27,
    "asset_root": "../../first_steps_rc1/난수범위_기초27/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "참"
    ],
    "expected_description": "기본 참 → 지정 수정 후 참.",
    "reason": "기존 Native 원문 보관을 지원합니다. 저장 ACK를 확인하고 정상 종료한 뒤 새 프로세스의 내 작업에서 재열어 명시 실행합니다. 보관 대상은 전체 원문·조건이며 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저 확인은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../first_steps_rc1/난수범위_기초27/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../first_steps_rc1/난수범위_기초27/앱.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "같은비율견주기_조사1",
  "title": "한국어 교과 · 같은 비율 견주기",
  "kind": "교과 탐구",
  "summary": "같은 고정 눈금에서 정확한 분수와 교차곱을 비교하고 분자를 직접 바꿉니다.",
  "tags": [
    "또니랑",
    "수학",
    "비율",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/같은비율견주기_조사1/뿌리.또니"
    },
    {
      "path": "비율견주기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/같은비율견주기_조사1/비율견주기.또니"
    }
  ],
  "course": {
    "id": "같은비율견주기_조사1",
    "title": "크기가 달라도 같은 비율일까?",
    "asset_root": "../../course_reading/같은비율견주기_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "2/3",
      "2/3",
      "참"
    ],
    "expected_description": "나분자4→5: 2/3 · 5/6 · 거짓",
    "reason": "Native 전체 원문 저장 ACK·정상 종료·새 프로세스 재열기·명시 실행을 확인했습니다. 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저·학생 검증은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/같은비율견주기_조사1/뿌리.또니"
    },
    {
      "path": "비율견주기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/같은비율견주기_조사1/비율견주기.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "비율견주기.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "비율견주기.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "행렬두열기여_조사1",
  "title": "한국어 교과 · 두 열의 기여",
  "kind": "교과 탐구",
  "summary": "같은 작업대에서 두 열의 기여와 변환 좌표를 비교하고 행렬 한 성분을 고칩니다.",
  "tags": [
    "또니랑",
    "수학",
    "행렬",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/행렬두열기여_조사1/전단/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../course_reading/행렬두열기여_조사1/전단/앱.또니"
    },
    {
      "path": "선형.또니",
      "role": "ddn_source",
      "url": "../../course_reading/행렬두열기여_조사1/전단/선형.또니"
    }
  ],
  "course": {
    "id": "행렬두열기여_조사1",
    "title": "두 열이 만드는 새 위치",
    "asset_root": "../../course_reading/행렬두열기여_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "2",
      "1",
      "1"
    ],
    "expected_description": "나칸1.0→0.0: 출력가로2→1, 출력세로1·행렬식1 유지",
    "reason": "Native 세 파일 전체 원문 저장 ACK·정상 종료·새 프로세스 재열기·명시 실행을 확인했습니다. 세계 진행·과거 출력은 보관하지 않습니다. 실제 브라우저·학생 검증은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/행렬두열기여_조사1/전단/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../course_reading/행렬두열기여_조사1/전단/앱.또니"
    },
    {
      "path": "선형.또니",
      "role": "ddn_source",
      "url": "../../course_reading/행렬두열기여_조사1/전단/선형.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        },
        {
          "path": "선형.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "세금부담_두실험_조사1",
  "title": "한국어 교과 · 세금과 대금",
  "kind": "교과 탐구",
  "summary": "모형 시장의 대금 몫을 비교하고 세금 또는 수요기울기 한 값만 직접 고칩니다.",
  "tags": [
    "또니랑",
    "교과",
    "경제",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/세금부담_두실험_조사1/A/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../course_reading/세금부담_두실험_조사1/A/앱.또니"
    },
    {
      "path": "시장.또니",
      "role": "ddn_source",
      "url": "../../course_reading/세금부담_두실험_조사1/A/시장.또니"
    }
  ],
  "course": {
    "id": "세금부담_두실험_조사1",
    "title": "세금을 누가 부담할까?",
    "asset_root": "../../course_reading/세금부담_두실험_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "55",
      "45",
      "45",
      "450",
      "5",
      "5",
      "0"
    ],
    "expected_description": "수요기울기1→3: 57.5·47.5·47.5·475·7.5·2.5·0",
    "reason": "실제 원문의 네 조건 계산·거래 장면과8관찰, Native 세 파일 전체 원문 ACK·정상 종료·새 프로세스 재열기·명시 실행을 확인했습니다. 세계 진행·과거 출력·실제 브라우저·학생 검증은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/세금부담_두실험_조사1/A/뿌리.또니"
    },
    {
      "path": "앱.또니",
      "role": "ddn_source",
      "url": "../../course_reading/세금부담_두실험_조사1/A/앱.또니"
    },
    {
      "path": "시장.또니",
      "role": "ddn_source",
      "url": "../../course_reading/세금부담_두실험_조사1/A/시장.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "앱.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "앱.또니",
          "role": "ddn_source"
        },
        {
          "path": "시장.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "사료목소리견주기_조사1",
  "title": "한국어 교과 · 사료의 목소리",
  "kind": "교과 탐구",
  "summary": "기사·인용·교사 재서술을 구별하며 표시 조건과 내 설명을 직접 고칩니다. 진실 점수를 발급하지 않습니다.",
  "tags": [
    "또니랑",
    "교과",
    "사료",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/사료목소리견주기_조사1/뿌리.또니"
    },
    {
      "path": "사료견주기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/사료목소리견주기_조사1/사료견주기.또니"
    }
  ],
  "course": {
    "id": "사료목소리견주기_조사1",
    "title": "같은 문자 계획을 왜 다르게 읽었을까?",
    "asset_root": "../../course_reading/사료목소리견주기_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [
      "4",
      "3",
      "1"
    ],
    "expected_description": "상소포함 참→거짓: 표시/직접기사/재서술4·3·1→3·2·1. 내설명은 해석이며 증명 아님.",
    "reason": "기존6조건과 학생 설명 수정의 실행·15관찰, Native 두 파일 전체 원문 ACK·정상 종료·새 프로세스 재열기·명시 실행을 확인했습니다. 세계 진행·과거 출력·실제 브라우저·학생 검증은 별도입니다."
  },
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/사료목소리견주기_조사1/뿌리.또니"
    },
    {
      "path": "사료견주기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/사료목소리견주기_조사1/사료견주기.또니"
    }
  ],
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "사료견주기.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "사료견주기.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "한칸씩길찾기_조사1",
  "title": "한국어 게임 · 한 칸씩 길찾기",
  "description": "오른쪽 누름마다 BFS 한 칸 확장. 원문 지도 수정과 두 관찰 표를 비교합니다.",
  "tags": [
    "또니랑",
    "게임",
    "길찾기",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/한칸씩길찾기_조사1/뿌리.또니"
    },
    {
      "path": "길찾기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/한칸씩길찾기_조사1/길찾기.또니"
    }
  ],
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/한칸씩길찾기_조사1/뿌리.또니"
    },
    {
      "path": "길찾기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/한칸씩길찾기_조사1/길찾기.또니"
    }
  ],
  "logicalInput": {
    "binding": "한칸씩길찾기/입력계약.json",
    "actions": "한칸씩길찾기/키배치.json",
    "capacity": 128
  },
  "course": {
    "id": "한칸씩길찾기_조사1",
    "title": "가까운 칸부터 길 찾기",
    "asset_root": "../../course_reading/한칸씩길찾기_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [],
    "expected_description": "명시 입력마다 한 칸 확장. 기본11/길이7, 고립4/도달 불가.",
    "reason": "실제 Worker·Native 입력과 원문 보관/별도프로세스 fresh 확인. 실제 브라우저·학생/Human 별도."
  },
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "길찾기.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "길찾기.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "벽피해도착하기_조사1",
  "title": "한국어 게임 · 벽을 피해 도착하기",
  "description": "방향 입력과 벽·경계·도착, 시도수와 이동수, 원문 벽 이동을 비교합니다.",
  "tags": [
    "또니랑",
    "게임",
    "미로",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/벽피해도착하기_조사1/뿌리.또니"
    },
    {
      "path": "미로.또니",
      "role": "ddn_source",
      "url": "../../course_reading/벽피해도착하기_조사1/미로.또니"
    }
  ],
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/벽피해도착하기_조사1/뿌리.또니"
    },
    {
      "path": "미로.또니",
      "role": "ddn_source",
      "url": "../../course_reading/벽피해도착하기_조사1/미로.또니"
    }
  ],
  "logicalInput": {
    "binding": "벽피해도착하기/입력계약.json",
    "actions": "한칸씩길찾기/키배치.json",
    "capacity": 128
  },
  "course": {
    "id": "벽피해도착하기_조사1",
    "title": "벽을 피해 도착하기",
    "asset_root": "../../course_reading/벽피해도착하기_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [],
    "expected_description": "방향을 눌러 이동하고 시도/이동/벽/경계를 구별합니다.",
    "reason": "현재 입력 패널·Worker·Native와 원문 저장/별도프로세스 fresh 확인, 세계 진행·브라우저·Human 별도."
  },
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "미로.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "미로.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});


FIRST_STEPS_PROJECTS.push({
  "id": "문지기와선택턴_조사1",
  "title": "한국어 게임 · 문지기와 선택 턴",
  "description": "공격·강타·방어·쉬기, 턴과 기력 거부, 반격피해 수정의 상태 변화를 비교합니다.",
  "tags": [
    "또니랑",
    "게임",
    "상태",
    "한국어"
  ],
  "sourceProfileIdentity": "v1-core-v25",
  "primaryView": "console",
  "viewGrade": "output-only",
  "files": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/문지기와선택턴_조사1/뿌리.또니"
    },
    {
      "path": "문지기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/문지기와선택턴_조사1/문지기.또니"
    }
  ],
  "nativeAuthoredFiles": [
    {
      "path": "뿌리.또니",
      "role": "root",
      "url": "../../course_reading/문지기와선택턴_조사1/뿌리.또니"
    },
    {
      "path": "문지기.또니",
      "role": "ddn_source",
      "url": "../../course_reading/문지기와선택턴_조사1/문지기.또니"
    }
  ],
  "logicalInput": {
    "binding": "문지기와선택턴/입력계약.json",
    "actions": "문지기와선택턴/조작대응.json",
    "capacity": 128
  },
  "course": {
    "id": "문지기와선택턴_조사1",
    "title": "어떤 선택을 다음 턴으로 가져갈까?",
    "asset_root": "../../course_reading/문지기와선택턴_조사1/",
    "student_doc": "학생용.html",
    "teacher_doc": "교사용.html",
    "expected": [],
    "expected_description": "선택 입력마다 상태를 비교합니다. 기력부족 거부는 턴/반격을 늘리지 않습니다.",
    "reason": "실제 Worker·Native 원래4입력과 공용패널/원문 수정·원문보관/별도process fresh 확인. 세계진행·브라우저·Human별도."
  },
  "portableSnapshot": {
    "source_profile_identity": "v1-core-v25",
    "local_package_payload": {
      "kind": "seamgrim.local_project_tree.v0",
      "root_path": "뿌리.또니",
      "entry_path": "문지기.또니",
      "files": [
        {
          "path": "뿌리.또니",
          "role": "root"
        },
        {
          "path": "문지기.또니",
          "role": "ddn_source"
        }
      ]
    }
  }
});
