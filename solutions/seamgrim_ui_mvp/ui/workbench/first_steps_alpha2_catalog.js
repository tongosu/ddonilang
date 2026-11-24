// WO14 education assets, not a language surface or an input tape format.
const response = await fetch(new URL('../../first_steps_alpha2/검증목록.json', import.meta.url));
if (!response.ok) throw new Error(`E_FIRST_STEPS_ASSET: 알파2 학습 목록을 읽을 수 없습니다 (${response.status}).`);
export const FIRST_STEPS_ALPHA2_MANIFEST = await response.json();
export const FIRST_STEPS_ALPHA2_PROJECTS = [
  ...FIRST_STEPS_ALPHA2_MANIFEST.cases, ...FIRST_STEPS_ALPHA2_MANIFEST.modifications,
].map(row => ({
  id:`first_steps_alpha2_${row.id}`, title:`첫걸음 ${row.id} · ${row.title}`,
  kind:'알파2 첫걸음', summary:'모양·마디·보개·입력·기록을 직접 탐구합니다.',
  tags:['첫걸음','알파2',String(row.lesson ?? row.id)],
  primaryView:row.view ? 'space2d' : 'console', viewGrade:'output-only',
  viewPolicy:{primary:'원문 실행 결과',side:'',overlay:'',magnifier:''},
  lessonDisposition:row.binding_file ? '명시 입력 계약으로 실행하기' : '예상하고 실행하기',
  files:[{path:'lesson.ddn',role:'main',url:`../../first_steps_alpha2/${row.file}`}],
  course:{...row,asset_root:'../../first_steps_alpha2/',reason:'본문의 손 계산·입력 순서와 비교하세요. 키보드 직접 실행은 제공된 입력 기록과 다른 실험입니다.'},
}));
