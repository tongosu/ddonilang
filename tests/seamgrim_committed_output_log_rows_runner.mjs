import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {extractObservationOutputLogFromState, extractObservationOutputRowsFromState}
  from '../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js';

// 실제 Rust output_log 항목의 기존 모양. 메타데이터를 출력 토큰으로 읽지 않는다.
const tokens=['table.row','0.1','p','10.544','econ',
  'table.row','0.1','qd','100','econ',
  'table.row','0.1','qs','32','econ',
  'table.row','0.1','gap','68','econ'];
const entries=tokens.map((text,index)=>({kind:'output',line_no:index+1,text,tick:0}));
const frame={schema:'seamgrim.engine_response.v0',tick_id:0,
  resources:{value_json:{output_log:entries}},channels:[],row:[]};
assert.deepEqual(extractObservationOutputLogFromState(frame).map(row=>row.text),tokens);
assert.deepEqual(extractObservationOutputRowsFromState(frame).map(({key,value})=>({key,value})),
  [{key:'p',value:'10.544'},{key:'qd',value:'100'},{key:'qs',value:'32'},{key:'gap',value:'68'}]);
assert.deepEqual(extractObservationOutputRowsFromState({...frame,state:{resources:frame.resources}}),
  extractObservationOutputRowsFromState(frame),'상/하위 기존 응답 경로가 같은 관찰표를 낸다');
assert.equal(extractObservationOutputRowsFromState({...frame,resources:{value_json:{output_log:[]}}}).length,0);
const legacyTokens=['table.row','0.1','p','10.544','econ'];
const legacyWithWarning={...frame,
  channels:[{key:'보개_출력_줄들',dtype:'text',role:'state'}],row:[JSON.stringify(legacyTokens)],
  resources:{value_json:{output_lines:legacyTokens,
    console_log:[{kind:'warn',line_no:1,text:'확인 안내',tick:0}]}},
};
assert.deepEqual(extractObservationOutputRowsFromState(legacyWithWarning).map(({key,value})=>({key,value})),
  [{key:'p',value:'10.544'}],'다른 console_log 경고가 기존 출력줄을 가리지 않는다');

if(process.env.DDN_WASM_DIR){
  const dir=path.resolve(process.env.DDN_WASM_DIR);
  const wasm=await import(pathToFileURL(path.join(dir,'ddonirang_tool.js')).href);
  wasm.initSync({module:fs.readFileSync(path.join(dir,'ddonirang_tool_bg.wasm'))});
  for(const [id,names] of [
    ['econ_supply_demand_seed_v1',['p','qd','qs','gap']],
    ['physics_heat_diffusion_seed_v1',['T1','T2','T3','meanT']],
  ]){
    const source=fs.readFileSync(path.join('solutions/seamgrim_ui_mvp/lessons_v25_alpha1',id,'lesson.ddn'),'utf8');
    const vm=new wasm.DdnWasmVm('');
    try{
      vm.update_logic_with_source_profile(source,'v1-core-v25');
      const raw=vm.step_one(),actual=JSON.parse(raw);
      const log=extractObservationOutputLogFromState(actual);
      const rows=extractObservationOutputRowsFromState(actual);
      assert.deepEqual(rows.map(row=>row.key),names,`${id}: 실제 WASM 출력의 네 관찰 이름`);
      for(let index=0;index<names.length;index++){
        assert.equal(rows[index].value,log[index*5+3].text,`${id}: 실행기 출력 그대로 표시`);
      }
      console.log(`ACTUAL_WASM_OUTPUT_ROWS_PASS ${id}`);
    }finally{vm.free();}
  }
}else console.log('ACTUAL_WASM_OUTPUT_ROWS_NOT_RUN');
console.log('COMMITTED_OUTPUT_LOG_ROWS_PASS');
