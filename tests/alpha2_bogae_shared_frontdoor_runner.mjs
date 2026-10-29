import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

// Actual product frontdoors; no source lowering, receipt-file ingress or new tape.
assert.ok(process.env.DDN_WASM_DIR, 'source-built isolated WASM directory required');
assert.ok(process.env.DDN_NATIVE_DIR, 'source-built native CLI directory required');
const wasmDir = path.resolve(process.env.DDN_WASM_DIR);
const wasmBytes = fs.readFileSync(path.join(wasmDir, 'ddonirang_tool_bg.wasm'));
const wasm = await import(pathToFileURL(path.join(wasmDir, 'ddonirang_tool.js')).href);
await wasm.default({module_or_path: wasmBytes});
const profile = 'v1-core-v25';
const prefix = '보기 := () 표준보기.결과표. 기본 := (보기=보기) 표준보개.으뜸. ';
const cases = [
  {id:'magnifier-local-list-selection',source:'(대상:(수)차림) 확대:보기값:셈씨 := { 원 := (중심=차림[대상[0],대상[1]], 반지름=1) 표준장면.원. 장면 := (요소들=차림[원], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 확대 <- (장면=장면) 표준보기.장면. }. 기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 자세히 := (대상=차림[2,3], 보기법=확대) 표준보개.돋. 짜임 := (관계들=차림[기본,자세히]) 표준보개.짜임.',expected:[],projection:'ready'},
  {id:'magnifier-hidden-list-selection-rejected',source:'외부 <- 차림[2,3]. (대상:(수)차림) 확대:보기값:셈씨 := { x := 외부[0]. 확대 <- () 표준보기.결과표. }. 자세히 := (대상=차림[2,3], 보기법=확대) 표준보개.돋.',reject:'E_BOGAE_RECIPE_UNSEALED'},
  {id:'sealed-magnifier-target-recipe',source:'(대상:수) 확대:보기값:셈씨 := { 원 := (중심=차림[0,0], 반지름=대상) 표준장면.원. 장면 := (요소들=차림[원], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 확대 <- (장면=장면) 표준보기.장면. }. 기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 자세히 := (대상=2, 보기법=확대) 표준보개.돋. 짜임 := (관계들=차림[기본,자세히]) 표준보개.짜임.',expected:[],projection:'ready'},
  {id:'magnifier-fixed-view-rejected',source:'자세히 := (대상=2, 보기법=(() 표준보기.결과표)) 표준보개.돋.',reject:'E_BOGAE_RECIPE_UNSEALED'},
  {id:'magnifier-hidden-world-read',source:'외부 <- 2. (대상:수) 확대:보기값:셈씨 := { 반지름 <- 외부. 확대 <- () 표준보기.결과표. }. 자세히 := (대상=2, 보기법=확대) 표준보개.돋.',reject:'E_BOGAE_RECIPE_UNSEALED'},
  {id:'magnifier-transitive-world-read',source:'외부 <- 2. (대상:수) 보조:보기값:셈씨 := { 반지름 <- 외부. 보조 <- () 표준보기.결과표. }. (대상:수) 확대:보기값:셈씨 := { 확대 <- (대상=대상) 보조. }. 자세히 := (대상=2, 보기법=확대) 표준보개.돋.',reject:'E_BOGAE_RECIPE_UNSEALED'},
  {id:'magnifier-wrong-result-type',source:'(대상:수) 확대:수:셈씨 := { 확대 <- 대상. }. 자세히 := (대상=2, 보기법=확대) 표준보개.돋.',reject:'E_BOGAE_RECIPE_TYPE'},
  {id:'magnifier-side-effect',source:'(대상:수) 확대:보기값:셈씨 := { 대상 보여주기. 확대 <- () 표준보기.결과표. }. 자세히 := (대상=2, 보기법=확대) 표준보개.돋.',reject:'E_BOGAE_RECIPE_UNSEALED'},
  {id:'magnifier-wrong-parameter-type',source:'(대상:수) 확대:보기값:셈씨 := { 확대 <- () 표준보기.결과표. }. 자세히 := (대상="글", 보기법=확대) 표준보개.돋.',reject:'E_RUNTIME_TYPE_MISMATCH'},
  {id:'pure-local-container-snapshot', source:'(기준:수) 계산:수:셈씨 := { 목록 <- 차림[차림[기준, 2], 차림[3, 4]]. 묶음 <- (첫째:기준, 둘째:2). 계산 <- 묶음.첫째 + 목록[0][1]. }. (기준=3) 계산 보여주기.', expected:['5']},
  {id:'pure-local-container-failure', source:'계산:수:셈씨 := { 목록 <- 차림[1, 1 / 0]. 계산 <- 9. }. () 계산 보여주기.', reject:'0으로 나눌 수 없습니다'},
  {id:'nested-coordinate-search', source:'칸들 <- 차림[차림[0, 1], 차림[2, 3]]. ((칸들, 차림[2, 3]) 들어있나) 보여주기. ((칸들, 차림[2, 3]) 찾아보기) 보여주기. ((칸들, 차림[3, 2]) 들어있나) 보여주기. ((칸들, 차림[3, 2]) 찾아보기) 보여주기.', expected:['참', '1', '거짓', '-1']},
  // RD-071 recursive exact equality; these are original product inputs.
  {id:'nested-list-exact-equality', source:'판 <- 차림[차림[1, 2], 차림[3, 4]]. (판 == 판) 보여주기. (판 != 차림[차림[1, 2], 차림[4, 3]]) 보여주기.', expected:['참', '참']},
  {id:'list-order-and-length-equality', source:'(차림[1, 2] == 차림[2, 1]) 보여주기. (차림[1, 2] == 차림[1]) 보여주기.', expected:['거짓', '거짓']},
  // A2-D7 feasibility only: existing RD-352/355/356 chained selection.
  // No grid/game callable, implicit broadcast or new source surface.
  {id:'nested-board-persistent-write', source:'판 <- 차림[차림[0, 0], 차림[0, 0]]. 이전 <- 판. 판[1][0] <- 7. (이전[1][0]) 보여주기. (판[1][0]) 보여주기. (판[0][0]) 보여주기.', expected:['0', '7', '0']},
  {id:'nested-board-out-of-range', source:'판 <- 차림[차림[0, 0], 차림[0, 0]]. 판[2][0] <- 7.', reject:'FATAL:CHARIM_INDEX_OUT_OF_RANGE'},
  {id:'nested-board-not-multiaxis', source:'판 <- 차림[차림[0, 0], 차림[0, 0]]. (판[1, 0]) 보여주기.', reject:'파싱 실패: ]'},
  {id:'result-table', source:'보기:보기값 := () 표준보기.결과표. 보기 보여주기.', expected:['<보기값:결과표>']},
  {id:'circle-text-plane-view',source:'원 := (중심=차림[0,0], 반지름=2) 표준장면.원. 글 := (자리=차림[0,1], 내용="중심") 표준장면.글. 장면 := (요소들=차림[원,글], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 보기 := (장면=장면) 표준보기.장면.',expected:[],projection:'ready'},
  {id:'circle-nonpositive-radius',source:'원 := (중심=차림[0,0], 반지름=0) 표준장면.원.',reject:'E_SCENE_RADIUS'},
  {id:'polygon-plane-view',source:'네모 := (꼭짓점들=차림[차림[0,0],차림[2,0],차림[2,2],차림[0,2]]) 표준장면.다각형. 장면 := (요소들=차림[네모], 가로범위=차림[-1,3], 세로범위=차림[-1,3]) 표준장면.평면. 보기 := (장면=장면) 표준보기.장면.',expected:[],projection:'ready'},
  {id:'polygon-composition-copy-cost',source:'네모 := (꼭짓점들=차림[차림[0,0],차림[2,0],차림[2,2],차림[0,2]]) 표준장면.다각형. 장면 := (요소들=차림[네모], 가로범위=차림[-1,3], 세로범위=차림[-1,3]) 표준장면.평면. 기본 := (보기=((장면=장면) 표준보기.장면)) 표준보개.으뜸. 짜임 := (관계들=차림[기본]) 표준보개.짜임.',expected:[],projection:'ready',compareVisitsTo:'polygon-plane-view',extraVisits:13},
  {id:'polygon-crossed',source:'도형 := (꼭짓점들=차림[차림[0,0],차림[2,2],차림[0,2],차림[2,0]]) 표준장면.다각형.',reject:'E_SCENE_GEOMETRY'},
  {id:'polygon-foldback',source:'도형 := (꼭짓점들=차림[차림[0,0],차림[2,0],차림[1,0],차림[2,2]]) 표준장면.다각형.',reject:'E_SCENE_GEOMETRY'},
  {id:'polygon-collinear',source:'도형 := (꼭짓점들=차림[차림[0,0],차림[1,0],차림[2,0]]) 표준장면.다각형.',reject:'E_SCENE_GEOMETRY'},
  {id:'polygon-wrong-coordinate',source:'도형 := (꼭짓점들=차림[차림[0,0,0],차림[2,0],차림[2,2]]) 표준장면.다각형.',reject:'E_SCENE_COORDINATE'},
  {id:'plane-reversed-bounds',source:'원 := (중심=차림[0,0], 반지름=2) 표준장면.원. 장면 := (요소들=차림[원], 가로범위=차림[5,-5], 세로범위=차림[-5,5]) 표준장면.평면.',reject:'E_SCENE_BOUNDS'},
  {id:'scene-view-not-a-record',source:'보기 := (장면=(가로:5)) 표준보기.장면.',reject:'E_SCENE_ARGUMENT'},
  {id:'circle-coordinate-wrong-dimension',source:'원 := (중심=차림[0,0,0], 반지름=2) 표준장면.원.',reject:'E_SCENE_COORDINATE'},
  {id:'scene-comparison-not-marker-equality',source:'원 := (중심=차림[0,0], 반지름=2) 표준장면.원. (원 == 원) 보여주기.',reject:'E_SCENE_OPERATION_UNSUPPORTED'},
  {id:'inline-primary-projection', source:'짜임:보개짜임값 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임.', expected:[], projection:'ready'},
  {id:'primary-committed-observation', source:'짜임:보개짜임값 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임. (측정: 3) 보임.', expected:[], projection:'ready', cell:'3'},
  {id:'primary', source:'보기:보기값 := () 표준보기.결과표. 기본:으뜸보개관계값 := (보기=보기) 표준보개.으뜸. 기본 보여주기.', expected:['<으뜸보개관계값>']},
  {id:'ordered-relations', source:prefix + '관계들 := 차림[기본, 기본].', expected:[]},
  // Owner approval 2026-10-08, RD-123/131/139: construction remains valid,
  // but two root candidates without an explicit entry cannot activate.
  {id:'composition', source:prefix + '짜임:보개짜임값 := (관계들=차림[기본]) 표준보개.짜임. 짜임 보여주기.', reject:'E_BOGAE_SELECTION_REQUIRED'},
  {id:'multiple-views-before-world-commit', source:'값 <- 9. 첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표.', reject:'E_BOGAE_SELECTION_REQUIRED'},
  {id:'overlay-side-projection', source:'기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 겹:겹보개관계값 := (보기=(() 표준보기.결과표), 기준=기본, 순서=1) 표준보개.겹. 곁:곁보개관계값 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁. 짜임 := (관계들=차림[기본,겹,곁]) 표준보개.짜임. (측정:3) 보임.', expected:[],projection:'ready',cell:'3'},
  {id:'overlay-side-not-primary-type', source:'곁:으뜸보개관계값 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁.', reject:'E_RUNTIME_TYPE_MISMATCH'},
  {id:'overlay-wrong-basis', source:'곁 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁. 겹 := (보기=(() 표준보기.결과표), 기준=곁, 순서=1) 표준보개.겹.', reject:'E_BOGAE_OVERLAY_BASIS_INVALID'},
  {id:'composition-primary-count', source:prefix + '짜임 := (관계들=차림[기본, 기본]) 표준보개.짜임.', reject:'E_BOGAE_PRIMARY_COUNT'},
  {id:'composition-nested', source:prefix + '짜임 := (관계들=차림[차림[기본]]) 표준보개.짜임.', reject:'E_BOGAE_UNSUPPORTED'},
  {id:'composition-wrong-element', source:prefix + '짜임 := (관계들=차림["으뜸"]) 표준보개.짜임.', reject:'E_BOGAE_UNSUPPORTED'},
  {id:'composition-unresolved-equality', source:prefix + '짜임 := (관계들=차림[기본]) 표준보개.짜임. (짜임 == 짜임) 보여주기.', reject:'정체 해소 전'},
  // 생성3 + 첫번째 보호4/연산1 + 검색 보호5/연산2 = 15.
  // 보여주기는 Stmt::Show이며 내장 함수 인자 보호 검사를 추가로 부르지 않는다.
  {id:'metered-collection-guard', source:'목록 := 차림[10, 20, 30]. (목록) 첫번째 보여주기. (목록, 20) 들어있나 보여주기.', expected:['10', '참'], visits:15},
  {id:'fake-view', source:'보기:보기값 := "결과표".', reject:'보기값'},
  {id:'fake-primary-view', source:'기본 := (보기="결과표") 표준보개.으뜸.', reject:'보기값'},
  {id:'unresolved-equality', source:prefix + '(기본 == 기본) 보여주기.', reject:'정체 해소 전'},
  {id:'unresolved-set', source:prefix + '결과 := (기본, 기본) 모음.', reject:'E_BOGAE_RELATION_UNRESOLVED'},
  {id:'unresolved-map-key', source:prefix + '결과 := (기본, 1) 짝맞춤.', reject:'E_BOGAE_RELATION_UNRESOLVED'},
  {id:'unresolved-search', source:prefix + '결과 := (차림[기본], 기본) 들어있나.', reject:'E_BOGAE_RELATION_UNRESOLVED'},
  {id:'unresolved-nested-set', source:prefix + '결과 := (차림[기본], 차림[기본]) 모음.', reject:'E_BOGAE_RELATION_UNRESOLVED'},
];
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'ddn-alpha2-bogae-parity-'));
const receipts=new Map();
for (const c of cases) {
  const sourceFile = path.join(directory, `${c.id}.ddn`);
  fs.writeFileSync(sourceFile, c.source, 'utf8');
  let summary, failure;
  try {
    summary = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(c.source, sourceFile, 1, profile));
  } catch (error) { failure = String(error); }
  if (c.reject) {
    assert.ok(failure?.includes(c.reject), `${c.id}: WASM expected ${c.reject}, got ${failure}`);
    assert.equal(summary, undefined, `${c.id}: failure must not publish a summary`);
  } else {
    assert.equal(failure, undefined, `${c.id}: ${failure}`);
    assert.deepEqual(summary.output_log_texts, c.expected, `${c.id}: independent expected output`);
    if (c.visits !== undefined) {
      assert.equal(summary.work_cost_receipt.counters.collection_tensor_element_visits,
        c.visits, `${c.id}: independently counted guard and operation visits`);
    }
    if(c.compareVisitsTo) {
      // Independently counted: primary copy5+list1+composition item1+
      // view copy5+generic argument-list visit1=13. Not a copied output oracle.
      assert.equal(summary.work_cost_receipt.counters.collection_tensor_element_visits
        -receipts.get(c.compareVisitsTo).counters.collection_tensor_element_visits,c.extraVisits);
    }
    receipts.set(c.id,summary.work_cost_receipt);
  }
  for (const alias of ['ddonirang', 'ddonilang', '또니랑']) {
    const run = spawnSync(path.join(process.env.DDN_NATIVE_DIR, `${alias}.exe`),
      ['실행', sourceFile, '--source-profile', profile, '--마디수', '1', '--json'],
      {encoding:'utf8', windowsHide:true, timeout:20000});
    assert.ifError(run.error);
    if (c.reject) {
      assert.notEqual(run.status, 0, `${alias}/${c.id}: invalid input accepted`);
      assert.ok((run.stdout + run.stderr).includes(c.reject), `${alias}/${c.id}: ${run.stdout}${run.stderr}`);
    } else {
      assert.equal(run.status, 0, `${alias}/${c.id}: ${run.stderr}`);
      const native = JSON.parse(run.stdout);
      const values = native.resources.value_json;
      assert.deepEqual((values.output_log ?? []).map(row => row.text), c.expected, `${alias}/${c.id}: output`);
      assert.equal(native.state_hash, summary.state_hash, `${alias}/${c.id}: world hash`);
      assert.deepEqual(native.bogae_projection, summary.bogae_projection,
        `${alias}/${c.id}: shared Rust projection bytes`);
      if (c.projection) assert.equal(summary.bogae_projection.status, c.projection);
      if (c.cell) assert.equal(summary.public_observation_table_projection.tables[0].rows[0].cells[0].value_text,c.cell);
      assert.deepEqual(native.work_cost_receipt, summary.work_cost_receipt,
        `${alias}/${c.id}: shared work-cost receipt`);
      for (const name of ['보기', '기본', '관계들', '짜임']) {
        assert.equal(values[name], undefined, `${alias}/${c.id}: authored carrier leaked into world resources`);
      }
    }
  }
  console.log(JSON.stringify({id:c.id, source_sha256:createHash('sha256').update(c.source).digest('hex'),
    state_hash:summary?.state_hash ?? null, expected:c.expected ?? c.reject, result:'PASS'}));
}
console.log(JSON.stringify({result:'ALPHA2_BOGAE_SHARED_FRONTDOOR_PASS', cases:cases.length,
  native_aliases:3, actual_wasm:1, wasm_sha256:createHash('sha256').update(wasmBytes).digest('hex'),
  source_fixture_directory:directory, composition_ui:'NOT_RUN', scope:'typed constructors, unresolved admission and existing nested-collection feasibility only'}));
