import {nativeSourceAvailable,nativeSourceRequest} from './native_source_transport.js';

const stable=value=>JSON.stringify(value,(_key,part)=>part&&typeof part==='object'&&!Array.isArray(part)
  ?Object.fromEntries(Object.entries(part).sort(([a],[b])=>a<b?-1:a>b?1:0)):part);

// Explicit cold request only. This never starts a VM, supplies input frames,
// retries a failed finish, or interprets a display summary as an execution permit.
export async function recoverNativeLogicalFinish({authored,binding,unchanged,
  available=nativeSourceAvailable,request=nativeSourceRequest}){
  unchanged();
  if(!await available())throw new Error('E_NATIVE_FINISH_RECOVERY_UNSUPPORTED: Native에 저장한 기록만 확인·복구할 수 있습니다.');
  unchanged();
  const recovered=await request('logical_finish_recover',authored,{input_request:{binding,snapshots:[]}});
  unchanged();
  if(typeof recovered?.original_output_json_utf8!=='string'
      ||typeof recovered?.notice?.excluded_input_count!=='string'
      ||!(recovered.original_failure===null||typeof recovered.original_failure==='string'))
    throw new Error('E_NATIVE_FINISH_RECOVERY_RESPONSE: 복구 응답을 확인하지 못했습니다.');
  const record=JSON.parse(recovered.original_output_json_utf8);
  if(!record||Object.keys(record).length!==2||!Array.isArray(record.tape_bytes)||!record.summary
      ||record.tape_bytes.some(byte=>!Number.isInteger(byte)||byte<0||byte>255))
    throw new Error('E_NATIVE_FINISH_RECOVERY_RESPONSE: 원래 완료 응답을 표시할 수 없습니다.');
  return recovered;
}

// Only transports the fenced physical queue. Native reconstructs and admits
// its own producer; neither the callback nor a matching summary is authority.
export function createNativeLogicalFinishBarrier({authored,binding,unchanged,
  available=nativeSourceAvailable,request=nativeSourceRequest,onCheckpoint=()=>{},onProgress=()=>{},checkpointMatches=()=>false}){
  let handle=null,nativePacket=null,originalPacket=null,preparedRecord=null,registration=null,terminalKind=null;
  const openProgress=async(summary,result)=>{
    if(handle!==null)return;
    if(!Array.isArray(summary?.frames)||summary.frames.length!==summary.madi_count||summary.outcome==='ABORTED')
      throw new Error('E_NATIVE_FINISH_PREFIX: 정상 처리한 마디가 필요합니다.');
    if(!/^sha256:[a-f0-9]{64}$/.test(result?.result_identity??''))
      throw new Error('E_INPUT_FINISH_NOT_READY: 원 값·형식의 결과 정체가 필요합니다.');
    // A UI hint only selects this check; it cannot admit a saved checkpoint.
    // The actual Native owner reopens the protected current world and supplies
    // its result identity. The existing full preview/commit comparison below
    // is still required before any Finished output can be published.
    if(checkpointMatches(summary)===true){
      onProgress('저장한 입력과 실행 상태를 확인하는 중입니다.');
      const began=await request('logical_finish_begin',authored,{input_request:{binding,snapshots:[]}});
      if(typeof began.session_handle!=='string')throw new Error('E_NATIVE_FINISH_SESSION: 종료 생산자가 없습니다.');
      handle=began.session_handle;registration=began.registration;
      unchanged();
      const actual=await request('logical_recording_result_identity',undefined,{session_handle:handle});
      unchanged();
      if(stable(actual)===stable(result))return;
      // Only an explicit different result takes the ordinary save path. An
      // admission, I/O, custody or uncertain response error is never a hint
      // to retry, replace a checkpoint, or bypass recovery.
      await request('logical_finish_close',undefined,{session_handle:handle});
      handle=null;registration=null;unchanged();
    }
    onProgress('성공한 마디를 저장하는 중입니다. 완료 전에는 창을 닫지 마세요.');
    const saved=await request('logical_save',authored,{input_request:{binding,
      snapshots:summary.frames.map(frame=>frame.input)},state_hash:summary.state_hash,expected_result_identity:result.result_identity});
    unchanged();
    if(saved.kind!=='native_logical_world'||stable(saved.summary)!==stable(summary))
      throw new Error('E_NATIVE_FINISH_PREFIX: 실제 저장 결과가 화면과 다릅니다.');
    onCheckpoint(saved.authoring_selection);
    onProgress('저장한 입력과 실행 상태를 확인하는 중입니다.');
    const began=await request('logical_finish_begin',authored,{input_request:{binding,snapshots:[]}});
    if(typeof began.session_handle!=='string')throw new Error('E_NATIVE_FINISH_SESSION: 종료 생산자가 없습니다.');
    handle=began.session_handle;registration=began.registration;
    unchanged();
  };
  const finish=async(packet,preview,result)=>{
    unchanged();
    if(terminalKind==='stop')throw new Error('E_NATIVE_RECORDING_NO_END: 중단 기록을 완료로 바꿀 수 없습니다.');
    if(!await available()){unchanged();return preview;}
    unchanged();
    if(handle===null){
      await openProgress(preview?.summary,result);
      if(registration?.accepted_event_count!==packet.first_unprocessed
        ||typeof registration?.producer_generation!=='string')
        throw new Error('E_NATIVE_FINISH_BEFORE_PENDING: 실제 입력 생산자의 처리 위치가 다릅니다.');
      originalPacket=stable(packet);
      nativePacket=JSON.stringify({...packet,producer_generation:registration.producer_generation});
      const candidate=await request('logical_finish_prepare',undefined,{session_handle:handle,pending_json_utf8:nativePacket});
      unchanged();
      preparedRecord=structuredClone(candidate);
      if(stable(candidate)!==stable(preview))throw new Error('E_NATIVE_FINISH_BEFORE_PENDING: E_NATIVE_FINISH_RESULT_MISMATCH: Native와 화면의 기록 후보가 다릅니다.');
    }
    if(originalPacket!==stable(packet))throw new Error('E_NATIVE_FINISH_PENDING_CHANGED: 원래 입력 queue를 보존하세요.');
    if(preparedRecord===null||stable(preparedRecord)!==stable(preview))
      throw new Error('E_NATIVE_FINISH_RESULT_MISMATCH: 원래 확인한 기록 후보를 보존하세요.');
    const nativeResult=await request('logical_recording_result_identity',undefined,{session_handle:handle});
    unchanged();
    if(stable(nativeResult)!==stable(result))throw new Error('E_NATIVE_FINISH_RESULT_IDENTITY_MISMATCH: 원 값·형식의 결과가 다릅니다.');
    onProgress('완료 기록을 보존하는 중입니다. 아직 완료되지 않았습니다.');
    const completed=await request('logical_finish_commit',undefined,{session_handle:handle,pending_json_utf8:nativePacket});
    unchanged();
    if(stable(completed)!==stable(preview))throw new Error('E_NATIVE_FINISH_RESULT_MISMATCH: 확정 기록이 화면과 다릅니다.');
    const notice=await request('logical_finish_notice',undefined,{session_handle:handle});
    unchanged();
    if(notice?.excluded_input_count!==String(packet.events.length))
      throw new Error('E_NATIVE_FINISH_NOTICE: 제외한 입력 수를 확인하지 못했습니다.');
    terminalKind='finish';
    return completed;
  };
  const barrier=async(packet,preview,result)=>{
    try{return await finish(packet,preview,result);}
    catch(error){
      if(String(error?.message??error).startsWith('E_NATIVE_FINISH_BEFORE_PENDING:')){
        // Only an actual pre-write rejection releases this transient owner.
        // Pending/uncertain/Finished evidence is never reset or overwritten.
        try{await barrier.close();}
        catch(closeError){throw new Error(`E_NATIVE_FINISH_RELEASE_UNCERTAIN: ${String(closeError?.message??closeError)}`);}
        nativePacket=null;originalPacket=null;preparedRecord=null;registration=null;
      }
      throw error;
    }
  };
  barrier.close=async()=>{
    if(handle!==null){await request('logical_finish_close',undefined,{session_handle:handle});handle=null;}
  };
  barrier.verifyReceipt=async()=>{
    unchanged();
    const verified=handle!==null
      ?await request('logical_finish_verify',undefined,{session_handle:handle})
      :await request('logical_verify_receipt',authored,{input_request:{binding,snapshots:[]}});
    unchanged();return verified;
  };
  barrier.stop=async(summary,result,failedInput=null)=>{
    unchanged();
    if(terminalKind==='finish')throw new Error('E_NATIVE_RECORDING_FINISHED: 완료 기록을 중단 기록으로 바꾸지 않습니다.');
    if(!await available()){unchanged();return null;}
    if(summary?.outcome==='ABORTED'){
      if(failedInput===null){unchanged();return null;}
      if(!Array.isArray(summary.frames)||summary.frames.length!==summary.madi_count
          ||!/^sha256:[a-f0-9]{64}$/.test(result?.result_identity??''))throw new Error('E_NATIVE_FAILURE_PREFIX');
      // No save of failed RAM state and no status projection to Running.
      // Open the already-existing real Native saved anchor K; Native checks
      // its accepted prefix, executes any suffix, then the actual bad input.
      if(handle===null){
        onProgress('저장한 시점에서 같은 입력의 실패를 확인하는 중입니다.');
        const began=await request('logical_finish_begin',authored,{input_request:{binding,snapshots:[]}});
        unchanged();if(typeof began.session_handle!=='string')throw new Error('E_NATIVE_FAILURE_ANCHOR_REQUIRED');
        handle=began.session_handle;registration=began.registration;
      }
      const aborted=await request('logical_recording_failed_input',undefined,{session_handle:handle,
        input_request:{binding,snapshots:summary.frames.map(frame=>frame.input)},failed_input:failedInput,
        expected_prefix_result_identity:result.result_identity});
      unchanged();
      if(aborted?.outcome!=='ABORTED'||stable(aborted.summary)!==stable(summary))throw new Error('E_NATIVE_FAILURE_RESULT_MISMATCH');
      terminalKind='stop';return aborted;
    }
    unchanged();await openProgress(summary,result);
    if(terminalKind!=='stop'){
      const actual=await request('logical_recording_result_identity',undefined,{session_handle:handle});
      unchanged();
      if(stable(actual)!==stable(result))throw new Error('E_NATIVE_NO_END_RESULT_IDENTITY_MISMATCH: 중단할 원 값·형식의 결과가 다릅니다.');
    }
    onProgress('미완 기록을 보존하는 중입니다. 아직 완료되지 않았습니다.');
    const stopped=await request('logical_recording_stop',undefined,{session_handle:handle});
    unchanged();
    if(stopped?.outcome!=='INCOMPLETE'||stable(stopped.summary)!==stable(summary))
      throw new Error('E_NATIVE_NO_END_RESULT: 실제 중단 보존 결과가 다릅니다.');
    terminalKind='stop';return stopped;
  };
  return barrier;
}
