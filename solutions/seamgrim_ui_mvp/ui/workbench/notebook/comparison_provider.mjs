import {comparisonSupport} from './comparison.mjs';

// Work draft: one actual provider instance, one fixed runtime connection.
// Its existing inspect() must perform source/project admission. No new wire,
// dependency resolver, model recognition or calculation is introduced here.
export function bindComparisonProvider(provider) {
  if(typeof provider?.inspect!=='function' || typeof provider?.project!=='function') {
    throw new Error('실제 원문 검사와 투영을 함께 공급하는 연결이 필요합니다.');
  }
  const pending=new Map(), admitted=new Map();
  let disposed=false,epoch=0;
  const fail=()=>({supported:false,reason:'각 원문의 실제 계산과 표시 관측 결속이 없어 비교를 연결하지 않았습니다.'});
  function live(){if(disposed)throw new Error('닫힌 관측 연결입니다.');}
  return {
    async inspect(source){
      live();
      const ticket=epoch;
      const report=await provider.inspect(source);
      live();
      if(ticket!==epoch)throw new Error('관측 연결이 바뀌어 이전 응답을 보관하지 않았습니다.');
      if(!report || typeof report!=='object')throw new Error('실제 관측 공급이 없습니다.');
      // Keep the exact admitted report for this invocation, rather than using
      // a later result cached under a matching model name or input summary.
      pending.set(source,{report:structuredClone(report),text:JSON.stringify(report)});
      return report;
    },
    project(report,source){
      live();
      // Existing source-session/pair snapshots clone reports. Bind exact value
      // and source through this pending inspection, not JS object identity.
      const binding=pending.get(source);pending.delete(source);
      if(!binding || JSON.stringify(report)!==binding.text) {
        throw new Error('검사한 원문과 실제 관측이 달라 표시하지 않았습니다.');
      }
      const view=provider.project(binding.report,source);
      // Projection owns typed quantities, row/key correspondence and units.
      // Retain its exact output; a caller cannot forge these with matching keys.
      admitted.set(source,JSON.stringify(view));
      return view;
    },
    compare(views,sources){
      live();
      if(!Array.isArray(views) || views.length!==2 || !Array.isArray(sources) || sources.length!==2) return fail();
      if(views.some((view,i)=>!admitted.has(sources[i]) || admitted.get(sources[i])!==JSON.stringify(view)))return fail();
      return comparisonSupport(views);
    },
    retain(sources){
      live();const keep=new Set(sources);
      for(const source of admitted.keys())if(!keep.has(source))admitted.delete(source);
      for(const source of pending.keys())if(!keep.has(source))pending.delete(source);
    },
    reset(sources=[]){
      live();epoch++;pending.clear();const keep=new Set(sources);
      for(const source of admitted.keys())if(!keep.has(source))admitted.delete(source);
    },
    dispose(){disposed=true;epoch++;pending.clear();admitted.clear();},
  };
}
