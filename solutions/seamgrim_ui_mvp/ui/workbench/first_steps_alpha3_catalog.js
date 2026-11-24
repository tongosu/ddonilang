// WO14: education assets; no new language or input wire.
const response = await fetch(new URL('../../first_steps_alpha3/검증목록.json', import.meta.url));
if (!response.ok) throw new Error(`E_FIRST_STEPS_ASSET: 알파3 학습 목록을 읽을 수 없습니다 (${response.status}).`);
export const FIRST_STEPS_ALPHA3_MANIFEST = await response.json();
export const FIRST_STEPS_ALPHA3_PROJECTS = [...FIRST_STEPS_ALPHA3_MANIFEST.cases,...FIRST_STEPS_ALPHA3_MANIFEST.modifications].map(row=>({
  id:`first_steps_alpha3_${row.id}`,title:`첫걸음 ${row.id} · ${row.title}`,kind:'알파3 첫걸음',
  summary:'수·모형·비교·반응을 예상하고 실행하며 탐구합니다.',tags:['첫걸음','알파3',String(row.lesson)],
  primaryView:row.view?'space2d':'console',viewGrade:'output-only',
  viewPolicy:{primary:'원문 실행 결과',side:'',overlay:'',magnifier:''},
  lessonDisposition:row.binding_file?'명시 입력 계약으로 실행하기':'예상하고 실행하기',
  files:row.project_files?row.project_files.map(file=>({path:file.path,role:file.role,url:`../../first_steps_alpha3/${file.file}`})):[{path:'lesson.ddn',role:'main',url:`../../first_steps_alpha3/${row.file}`}],
  ...(row.project_files?{portableSnapshot:{source_profile_identity:'v1-core-v25',local_package_payload:{kind:'seamgrim.local_project_tree.v0',root_path:row.project_root,entry_path:row.project_entry,files:row.project_files.map(file=>({path:file.path,role:file.role}))}}}:{}),
  course:{...row,asset_root:'../../first_steps_alpha3/',
    student_pdf:row.lesson>=21?'학생용_첫걸음_21-28차시.pdf':'학생용_첫걸음_17-20차시.pdf',
    teacher_pdf:row.lesson>=21?'교사용_첫걸음_21-28차시.pdf':'교사용_안내와_해답.pdf',
    reason:row.reason??'본문의 독립 계산·모형 가정과 비교하세요. 상황을 바꾼 뒤 ▶로 새로 계산합니다.'},
}));
