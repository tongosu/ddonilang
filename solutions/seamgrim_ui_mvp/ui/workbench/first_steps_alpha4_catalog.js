// Alpha4 education candidates. Typed input names/codec remain provisional.
export const FIRST_STEPS_ALPHA4_PROJECTS=[
  {id:'first_steps_alpha4_29A',title:'첫걸음 29A · 초기값과 추 끌기',lab:'alpha3_double_pendulum_lab',file:'alpha3_double_pendulum_lab.ddn',lesson:29},
  {id:'first_steps_alpha4_30A',title:'첫걸음 30A · 조작 기록',lab:'alpha4_manipulation_point',file:'alpha4_manipulation_point.ddn',lesson:30},
  {id:'first_steps_alpha4_30B',title:'첫걸음 30B · LRL 종합 탐구',lab:'alpha4_lrl_lab',file:'alpha4_lrl_lab.ddn',lesson:30}
].map(row=>({...row,kind:'알파4 첫걸음',summary:'예상하고 조작하고 비교한 뒤 이유를 설명합니다.',tags:['첫걸음','알파4'],primaryView:'space2d',viewGrade:'explicit',
  files:[{path:'lesson.ddn',role:'main',url:`../../samples/${row.file}`}],
  course:{id:row.id.replace('first_steps_alpha4_',''),title:row.title,lesson:row.lesson,lab_id:row.lab,asset_root:'../../first_steps_alpha4/',student_pdf:'학생용_첫걸음_29-30차시.pdf',teacher_pdf:'교사용_첫걸음_29-30차시.pdf',
    expected_description:row.lesson===29?'초기값 편집은 명시 재실행 전 세계를 바꾸지 않습니다.':'실제 조작 기록을 같은 원문·계약으로 재생하면 입력·세계 해시가 일치합니다.',reason:'원문 편집·세계 입력·화면 조작과 모형의 가정을 구별합니다.'}}));
