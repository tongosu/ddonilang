// Catalog construction runs at the original workbench initialization point.
export function createWorkbenchProjectCatalog({ SOURCE_ANALYSIS_PROFILE_IDENTITY, FIRST_STEPS_PROJECTS, FIRST_STEPS_ALPHA3_PROJECTS, FIRST_STEPS_ALPHA4_PROJECTS }) {
const FALLBACK_PENDULUM_DDN = `(매마디)마다 {
  중력:수 := 9.8.
  줄길이:수 := 1.
  처음각:수 := 0.5.
  처음각 보여주기.
}.
`;

const FALLBACK_PROJECTILE_DDN = `(매마디)마다 {
  초기속도x:수 := 8.
  초기속도y:수 := 15.
  중력가속도:수 := 9.8.
  시간간격:수 := 0.1.
  초기속도x 보여주기.
  초기속도y 보여주기.
  중력가속도 보여주기.
}.
`;

const PROJECTS = [
  {
    nativeAuthoredFiles: [{path:'root.ddn',role:'root',url:'../../minimal/native_calculation/root.ddn'}, {path:'main.ddn',role:'ddn_source',url:'../../minimal/native_calculation/main.ddn'}],
    id: "rc1_minimal_calculation", title: "첫 계산 · 2 + 3", kind: "RC1 최소 예제 · 한 번 실행",
    summary: "원문의 두 수를 바꾸고 실행하면 계산 결과가 바뀝니다.",
    tags: ["공식", "첫 계산", "RC1 최소판"], primaryView: "console", viewGrade: "output-only",
    viewPolicy: { primary: "실행 결과", side: "", overlay: "", magnifier: "" },
    files: [{ path: "lesson.ddn", role: "main", url: "../../minimal/first_calculation.ddn" }],
  },
  {
    nativeAuthoredFiles: [{path:'root.ddn',role:'root',url:'../../minimal/native_observation/root.ddn'}, {path:'main.ddn',role:'ddn_source',url:'../../minimal/native_observation/main.ddn'}],
    id: "rc1_minimal_observation", title: "값 바꾸고 관찰하기", kind: "RC1 최소 예제 · 한 번 실행",
    summary: "값을 수정하고 명시적으로 다시 실행해 실행 결과를 확인합니다.",
    tags: ["공식", "원문 편집", "관찰"], primaryView: "console", viewGrade: "output-only",
    viewPolicy: { primary: "실행 결과", side: "", overlay: "", magnifier: "" },
    files: [{ path: "lesson.ddn", role: "main", url: "../../minimal/change_and_observe.ddn" }],
  },
  {
    id: "rep_phys_projectile_xy_v1",
    title: "포물선 운동",
    kind: "공식 물리 DDN",
    summary: "원문이 위치·속도 화살표·시간별 x/y 표본·현재 위치 돋보기를 선언합니다. 지나온 자리는 별도의 화면 기록입니다.",
    tags: ["공식", "물리", "포물선", "WASM", "공개 쐐기"],
    primaryView: "projectile",
    viewGrade: "explicit",
    viewPolicy: {
      primary: "포물선 위치와 지나온 자리(화면 기록)",
      side: "가로속도/세로속도/중력 관찰값",
      overlay: "현재 속도 벡터(표시 배율 0.2)",
      magnifier: "현재 위치 돋보기",
    },
    files: [
      {
        path: "lesson.ddn",
        role: "main",
        url: "../../samples/alpha2_parabola_composition.ddn",
      },
    ],
  },
  {
    id: "alpha2_logical_tetris", title: "테트리스", kind: "논리 행동 · 키보드 작품",
    summary: "▶로 시작합니다. ←·→ 이동, ↓ 내리기, ↑ 회전, Space 떨어뜨리기, r 다시 시작. 보개에서 키를 누르세요.",
    tags: ["공식", "게임", "입력", "기록 재생"], primaryView: "space2d", viewGrade: "explicit",
    logicalInput: {binding: "alpha2_logical_tetris.binding.json", actions: "alpha2_logical_tetris.action_map.json", capacity: 128},
    nativeAuthoredFiles:[{path:'root.ddn',role:'root',url:'../../samples/beta1_logical_tetris/root.ddn'},
      {path:'main.ddn',role:'ddn_source',url:'../../samples/beta1_logical_tetris/main.ddn'}],
    viewPolicy: {primary: "칸판", overlay: "그림자 조각", side: "점수·다음 조각", magnifier: ""},
    files: [{path: "lesson.ddn", role: "main", url: "../../samples/alpha2_logical_tetris.ddn"}],
  },
  {
    id: "rep_phys_pendulum_wave1_v1",
    title: "진자 보개짜임",
    kind: "단일 DDN 파일",
    summary: "작은 각도 근사 진자를 관찰합니다. 원문이 추·각도 겹·각도와 에너지의 별도 곁·추 돋보기를 선언합니다.",
    tags: ["물리", "진자", "보개", "WASM"],
    primaryView: "pendulum",
    viewGrade: "explicit",
    viewPolicy: {
      primary: "진자 공간 보개",
      side: "각도·각속도·에너지 그래프",
      overlay: "각도와 수직 기준선",
      magnifier: "진자 추 위치 돋보기",
    },
    files: [
      {
        path: "lesson.ddn",
        role: "main",
        url: "../../samples/alpha2_pendulum_composition.ddn",
      },
    ],
  },
  {
    id: "local_linear_slope_v1",
    title: "일차함수 기울기 작업",
    kind: "한 번 실행",
    summary: "직선과 기울기 삼각형, x/y 표, 점 좌표를 원문으로 선언합니다. 시간이 흐르지 않으므로 한 번만 실행합니다.",
    tags: ["수학", "일차함수", "내 작업"],
    primaryView: "line",
    viewGrade: "explicit",
    viewPolicy: {
      primary: "그래프 보개",
      side: "x/y 관찰 표",
      overlay: "",
      magnifier: "",
    },
    files: [
      {
        path: "lesson.ddn",
        role: "main",
        url: "../../samples/alpha2_linear_composition.ddn",
      },
    ],
  },
  {
    id: "alpha3_function_lab",
    title: "함수 탐구 · 근과 넓이",
    kind: "알파3 탐구 · 한 번 실행",
    summary: "이분법의 근과 잔차, 사다리꼴 n칸·2n칸의 넓이와 오차를 함께 비교합니다. 원문의 이분횟수와 나눈수를 바꿔 실험합니다.",
    tags: ["수학", "탐구", "근 찾기", "적분", "오차", "알파3"],
    primaryView: "space2d", viewGrade: "explicit",
    viewPolicy: {primary:"함수의 표본",overlay:"계산표시",side:"계산비교",magnifier:""},
    files: [{path:"lesson.ddn",role:"main",url:"../../samples/alpha3_function_lab.ddn"}],
    exploration: {
      parameters:{이분횟수:{min:1,max:16,integer:true},나눈수:{min:1,max:32,integer:true},해석:{min:0,max:1,integer:true}},
      presets:[{label:"근 그림",values:{해석:0}},{label:"넓이 그림",values:{해석:1}},{label:"거칠게",values:{이분횟수:6,나눈수:4}},{label:"기본",values:{이분횟수:12,나눈수:8}},{label:"촘촘하게",values:{이분횟수:16,나눈수:16}}],
      guide:"근 그림은 x²−2가 0인 자리, 넓이 그림은 x²의 0~1 적분을 보여줍니다. 사다리꼴 겹을 꺼 곡선과 비교하세요. 단추는 초기값 원문만 바꿉니다. ▶로 다시 계산하고 곁의 잔차·넓이 오차를 읽습니다.",
    },
  },
  {
    id:"alpha3_market_lab", title:"작은 시장 · 복리와 세금", kind:"알파3 탐구 · 한 번 실행",
    summary:"단리와 복리, 선형 시장의 세전·세후 균형, 소비자와 생산자의 부담을 계산하고 비교합니다. 실제 시장을 예측하는 모형은 아닙니다.",
    tags:["경제","탐구","복리","세금","알파3"],primaryView:"space2d",viewGrade:"explicit",
    viewPolicy:{primary:"기준그림",overlay:"비교표시",side:"금액과부담",magnifier:""},
    files:[{path:"lesson.ddn",role:"main",url:"../../samples/alpha3_market_lab.ddn"}],
    exploration:{parameters:{세금:{min:0,max:16,integer:true},이자율:{min:0,max:0.1},해수:{min:0,max:10,integer:true},시장해석:{min:0,max:1,integer:true}},
      presets:[{label:"시장 그림",values:{시장해석:0}},{label:"단리·복리 그림",values:{시장해석:1}},{label:"10년 비교",values:{시장해석:1,이자율:0.1,해수:10}},{label:"세금 없음",values:{세금:0,이자율:0.05,해수:5}},{label:"기본 시장",values:{세금:8,이자율:0.05,해수:5}},{label:"높은 세금",values:{세금:16,이자율:0.05,해수:5}}],
      guide:"시장 그림은 Qd=100−5Pc, Qs=20+3Ps, Pc−Ps=세금인 작은 모형입니다. 단리·복리 그림은 연도별 금액을 천 단위로 비교합니다. 단추 뒤 ▶로 다시 계산하며 겹을 꺼 기준과 비교합니다. 두 그림의 숫자와 세계 해시는 같지만 서로 다른 현상이므로 인과관계로 읽지 마세요.",
    },
  },
  {
    id:"alpha3_double_pendulum_lab",title:"이중 진자 · 초기조건과 오차",kind:"알파3 탐구 · 마디 진행",
    summary:"두 추의 실제 결합 운동을 RK4로 계산합니다. 초기조건을 바꾼 점선과 간격을 반으로 줄인 곁을 구별해 관찰합니다.",
    tags:["물리","탐구","이중진자","오차","알파3"],primaryView:"space2d",viewGrade:"explicit",
    viewPolicy:{primary:"기준진자",overlay:"바꾼진자",side:"반간격진자·측정값",magnifier:""},
    files:[{path:"lesson.ddn",role:"main",url:"../../samples/alpha3_double_pendulum_lab.ddn"}],
    exploration:{parameters:{초기각1:{min:0,max:2},초기각2:{min:0,max:2},작은차이:{min:0,max:0.1},간격:{min:0.01,max:0.04}},
      presets:[{label:"정지",values:{초기각1:0,초기각2:0,작은차이:0,간격:0.02}},{label:"기본",values:{초기각1:1.2,초기각2:0.8,작은차이:0.001,간격:0.02}},{label:"큰 초기 차이",values:{초기각1:1.2,초기각2:0.8,작은차이:0.1,간격:0.02}},{label:"큰 각도 관찰",values:{초기각1:2,초기각2:1.8,작은차이:0.001,간격:0.02}}],
      guide:"질량·길이 1, 마찰 없는 모형입니다. 실선과 점선은 초기조건 비교이고 오른쪽 진자는 같은 초기조건에 반 간격을 쓴 계산입니다. 에너지 변화와 수치비교차를 함께 보세요. 간격은 원문 모형의 적분 간격이며 호스트 dt=1과 다릅니다.",
    },
  },
];

PROJECTS.push({
  id:"alpha4_manipulation_point",title:"점과 반지름 · 조작 기록",kind:"알파4 잠정 입력 실습",
  summary:"▶로 실행한 뒤 입력 기록을 열어 반지름과 좌표를 조작합니다. 실제 마디 입력을 저장해 다시 재생할 수 있습니다.",
  tags:["입력","기록","알파4"],primaryView:"space2d",viewGrade:"explicit",
  logicalInput:{binding:"alpha4_manipulation_point.binding.json",actions:"alpha4_manipulation_point.action_map.json",capacity:4096},
  files:[{path:"lesson.ddn",role:"main",url:"../../samples/alpha4_manipulation_point.ddn"}]
},{
  id:"alpha4_lrl_lab",title:"LRL · 궤도와 속도 원",kind:"알파4 탐구 · 잠정 후보",
  summary:"같은 운동의 궤도와 속도 공간을 나란히 봅니다. 초기값을 바꾼 뒤 ▶로 다시 실행하세요.",
  tags:["물리","탐구","LRL","알파4"],primaryView:"space2d",viewGrade:"explicit",
  files:[{path:"lesson.ddn",role:"main",url:"../../samples/alpha4_lrl_lab.ddn"}],
  exploration:{parameters:{이심률:{min:0,max:0.8},섭동:{min:0,max:0.05},속도배율:{min:0.7,max:1.3},간격:{min:0.005,max:0.02},발사설정:{min:0,max:1,integer:true,hidden:true}},
    presets:[{label:"원 운동",values:{이심률:0,섭동:0,속도배율:1,간격:0.01,발사설정:0}},
      {label:"타원",values:{이심률:0.5,섭동:0,속도배율:1,간격:0.01,발사설정:0}},
      {label:"섭동 비교",values:{이심률:0.5,섭동:0.02,속도배율:1,간격:0.01,발사설정:0}}],
    guide:"공을 위로 끌어 놓으면 끌기 벡터를 초기 발사 속도로 원문에 적습니다(vx ±3, vy 0.2~3). ▶로 명시 재실행하세요. 무차원 중앙힘 모형이며 섭동=0에서만 A² 검산과 LRL 보존이 성립합니다. 방향 변화는 초기 A에서 현재 A까지의 주각(rad); A가 거의 0이면 방향없음입니다. 궤적은 5마디마다 표본하며 반 간격이 고정소수 오차까지 항상 줄이지는 않습니다."}
});

PROJECTS.push(...[
  {
    "id": "v2_helper_producer",
    "title": "V2 첫걸음 · 고정 도우미 판본 만들기",
    "kind": "V2 차수 1a · 비공개 개발 예제",
    "summary": "안내 V2_STAGE1_FIRST_STEPS.md와 함께 사용합니다. 판본 발행은 기존 작품을 자동 갱신하지 않습니다.",
    "tags": [
      "공식",
      "V2",
      "첫걸음",
      "가지"
    ],
    "primaryView": "console",
    "viewGrade": "output-only",
    "files": [
      {
        "path": "root.ddn",
        "role": "root",
        "url": "../../samples/v2_stage1/helper_producer/root.ddn"
      },
      {
        "path": "app/main.ddn",
        "role": "ddn_source",
        "url": "../../samples/v2_stage1/helper_producer/app/main.ddn"
      },
      {
        "path": "lib/helper.ddn",
        "role": "ddn_source",
        "url": "../../samples/v2_stage1/helper_producer/lib/helper.ddn"
      }
    ],
    "portableSnapshot": {
      "source_profile_identity": "v1-core-v25",
      "local_package_payload": {
        "kind": "seamgrim.local_project_tree.v0",
        "root_path": "root.ddn",
        "entry_path": "app/main.ddn",
        "files": [
          {
            "path": "root.ddn",
            "role": "root"
          },
          {
            "path": "app/main.ddn",
            "role": "ddn_source"
          },
          {
            "path": "lib/helper.ddn",
            "role": "ddn_source"
          }
        ]
      }
    },
    "course": {
      "id": "V2-1a-1",
      "title": "V2 첫걸음 · 고정 도우미 판본 만들기",
      "reason": "판본은 고정됩니다. 새 판본을 발행해도 A/B는 각각 명시 갱신해야 바뀝니다.",
      "asset_root": "../../first_steps_rc1/v2_examples_r1/",
      "expected_description": "A7/B8 → A만9/B8 → B도10",
      "student_doc": "학생용_도우미판본.html",
      "teacher_doc": "교사용.html#helper"
    }
  },
  {
    "id": "v2_helper_consumer",
    "title": "V2 첫걸음 · A/B를 따로 갱신하기",
    "kind": "V2 차수 1a · 비공개 개발 예제",
    "summary": "안내 V2_STAGE1_FIRST_STEPS.md와 함께 사용합니다. 판본 발행은 기존 작품을 자동 갱신하지 않습니다.",
    "tags": [
      "공식",
      "V2",
      "첫걸음",
      "가지"
    ],
    "primaryView": "console",
    "viewGrade": "output-only",
    "files": [
      {
        "path": "root.ddn",
        "role": "root",
        "url": "../../samples/v2_stage1/helper_consumer/root.ddn"
      },
      {
        "path": "app/main.ddn",
        "role": "ddn_source",
        "url": "../../samples/v2_stage1/helper_consumer/app/main.ddn"
      }
    ],
    "portableSnapshot": {
      "source_profile_identity": "v1-core-v25",
      "local_package_payload": {
        "kind": "seamgrim.local_project_tree.v0",
        "root_path": "root.ddn",
        "entry_path": "app/main.ddn",
        "files": [
          {
            "path": "root.ddn",
            "role": "root"
          },
          {
            "path": "app/main.ddn",
            "role": "ddn_source"
          }
        ]
      }
    },
    "course": {
      "id": "V2-1a-2",
      "title": "V2 첫걸음 · A/B를 따로 갱신하기",
      "reason": "판본은 고정됩니다. 새 판본을 발행해도 A/B는 각각 명시 갱신해야 바뀝니다.",
      "asset_root": "../../first_steps_rc1/v2_examples_r1/",
      "expected_description": "A7/B8 → A만9/B8 → B도10",
      "student_doc": "학생용_도우미판본.html",
      "teacher_doc": "교사용.html#helper"
    }
  }
]);

// Visual examples and a separate, ordered first-steps course.
// Drawing helpers are real DDN module imports, not copied source or host lowering.
function commonDrawingProjectFiles(lab) {
  const paths=[`${lab}.project.ddn`,`${lab}.ddn`,'alpha4_drawing_common.ddn'];
  return {
    files:paths.map((path,index)=>({path,role:index===0?'root':'ddn_source',url:`../../samples/${path}`})),
    portableSnapshot:{source_profile_identity:SOURCE_ANALYSIS_PROFILE_IDENTITY,
      local_package_payload:{kind:'seamgrim.local_project_tree.v0',root_path:paths[0],entry_path:paths[1],
        files:paths.map((path,index)=>({path,role:index===0?'root':'ddn_source'}))}}
  };
}
for(const project of PROJECTS) {
  if(['alpha3_double_pendulum_lab','alpha4_lrl_lab'].includes(project.id))
    Object.assign(project,commonDrawingProjectFiles(project.id));
}
for (const project of [...FIRST_STEPS_ALPHA3_PROJECTS,...FIRST_STEPS_ALPHA4_PROJECTS]) {
  const lab=PROJECTS.find(item=>item.id===project.course.lab_id);
  if(lab) { project.exploration=lab.exploration; project.viewPolicy=lab.viewPolicy; project.logicalInput=lab.logicalInput;
    if(['alpha3_double_pendulum_lab','alpha4_lrl_lab'].includes(lab.id)) Object.assign(project,commonDrawingProjectFiles(lab.id)); }
}
for (const project of [...PROJECTS, ...FIRST_STEPS_PROJECTS]) project.sourceProfileIdentity = SOURCE_ANALYSIS_PROFILE_IDENTITY;

  return { PROJECTS, FALLBACK_PENDULUM_DDN, FALLBACK_PROJECTILE_DDN };
}
