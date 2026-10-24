// RC1 minimal composition: only examples authored for this publication slice.
export function createWorkbenchProjectCatalog({ SOURCE_ANALYSIS_PROFILE_IDENTITY }) {
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
  ].map(project => ({ ...project, sourceProfileIdentity: SOURCE_ANALYSIS_PROFILE_IDENTITY }));
  return { PROJECTS, FALLBACK_PENDULUM_DDN: "", FALLBACK_PROJECTILE_DDN: "" };
}
