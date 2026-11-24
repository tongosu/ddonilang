// Read-only local consumer. No new run, renderer, identity, persistence or math.
const RUN_FIELDS = ['schema','run_id','project_id','source_hash','project_source_hash',
  'source_kind','bogae_entry','authored_artifact_text','project_runtime_artifact_text',
  'recipe_hash','recipe_version','setup','parent_run_id','parent_madi','execution_source',
  'project_prepare_link_hash','project_graph_hash','linked_source_hash','final_status',
  'shared_runtime_result','work_cost_receipt','canonical_view_history'];
const FRAME_FIELDS = ['frame_id','madi','state_hash','source_hash','recipe_hash',
  'input_snapshot','wasm_state'];
const pick = (value, keys) => Object.fromEntries(keys.filter(k => value[k] !== undefined).map(k => [k,value[k]]));
const fail = message => { throw Error(message); };

export function createCompletedRunSelection({readRuns}) {
  const contexts = new WeakMap(), capturedRuns = new Set();
  let selected = null, revision = 0, listed = [];
  const fingerprint = run => JSON.stringify([run.run_id,run.project_id,run.source_hash,
    run.project_source_hash,run.final_status,run.frames.map(f=>[f.frame_id,f.madi,f.state_hash,f.source_hash,f.input_snapshot?.dtText])]);
  function capture(run, context) {
    // Presentation capture must never interrupt the existing execution/archive.
    try {
      if (!run || run.schema !== 'seamgrim.run_record.v1'
          || context?.project?.id !== run.project_id) return false;
      contexts.set(run, structuredClone({
        project: pick(context.project, ['id','name','primaryView']),
        viewConfig: pick(context.viewConfig ?? {}, ['primary_view','role','mode']),
        sourceProfile: String(context.sourceProfile ?? ''),
      }));
      capturedRuns.add(run);
      return true;
    } catch { return false; }
  }
  const runs = () => {
    const items = readRuns();
    return [...new Set([...capturedRuns,...(Array.isArray(items)?items:[])])].filter(r => r?.final_status === 'completed'
      && r.schema === 'seamgrim.run_record.v1' && Array.isArray(r.frames) && r.frames.length);
  };
  function list() {
    revision++;
    listed = runs().map(run=>({run, fingerprint:fingerprint(run)}));
    return listed.map(({run},index) => ({index, revision, runId:run.run_id, projectId:run.project_id,
      contextAvailable:contexts.has(run),
      frames:run.frames.map((frame,i) => ({index:i, madi:frame.madi, stateHash:frame.state_hash})),
    }));
  }
  function snapshot(choice) {
    const entry = listed[choice?.run], run = entry?.run;
    if (choice?.revision !== revision || !run || !runs().includes(run)
        || fingerprint(run) !== entry.fingerprint) fail('완료 실행 목록이 바뀌었습니다. 목록을 다시 읽고 선택하세요.');
    if (!run || !Number.isSafeInteger(choice?.frame) || choice.frame < 0
        || choice.frame >= run.frames.length) fail('선택한 완료 실행의 마디가 없습니다.');
    const context = contexts.get(run);
    if (!context) fail('이 실행 당시 작품·보기 설정이 없습니다. 새 실행을 명시적으로 완료한 뒤 선택하세요.');
    if (context.project.id !== run.project_id) fail('완료 실행의 작품 결속이 다릅니다.');
    const copied = structuredClone({...context, run:{...pick(run,RUN_FIELDS),
      frames:run.frames.map(frame => pick(frame,FRAME_FIELDS))}});
    // Keep the selected frame inside its cloned run: existing receipt consumers
    // use run.frames.indexOf(frame), not independently cloned equal objects.
    copied.frame = copied.run.frames[choice.frame];
    copied.frameIndex = choice.frame;
    copied.overlayReceiptAvailable = Boolean(copied.run.shared_runtime_result && copied.run.work_cost_receipt);
    copied.basic2D = 'existing_renderer_must_check_frame';
    return copied;
  }
  function choose(a,b) {
    if (a?.run === b?.run) fail('서로 다른 완료 실행 두 건을 선택하세요.');
    const A=snapshot(a), B=snapshot(b);
    const timeRows = s => s.run.frames.slice(0,s.frameIndex+1).map(frame => ({
      madi:frame.madi, dt:frame.input_snapshot?.dtText ?? null,
      receipt:frame.input_snapshot?.input_snapshot_sha256 ?? null,
    }));
    const ta=timeRows(A), tb=timeRows(B);
    const complete = rows => rows.every(r => typeof r.dt === 'string' && r.dt
      && /^[0-9a-f]{64}$/u.test(r.receipt ?? ''));
    selected={A,B,time:{
      status:complete(ta)&&complete(tb) ? 'input_receipts_present' : 'time_receipts_missing',
      sameMadi:A.frame.madi===B.frame.madi,
      sameDeliveredDtSequence:complete(ta)&&complete(tb)&&JSON.stringify(ta.map(r=>[r.madi,r.dt]))===JSON.stringify(tb.map(r=>[r.madi,r.dt])),
      physicalElapsedTimeEquality:'NOT_PRODUCED',
      note:'마디 수나 같은 화면 위치만으로 같은 작품 시간을 판정하지 않습니다.',
    }};
    return structuredClone(selected);
  }
  return {capture,list,choose,selected:()=>selected ? structuredClone(selected) : null};
}
