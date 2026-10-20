// B2-07 provisional local editing material only. Never a saved world,
// project identity issuer, receipt, runtime checkpoint or recovery permit.
const SCHEMA='seamgrim.workbench.edit_draft.b2-provisional-1';
const PREFIX=SCHEMA+':';
// A locator for the notice only, never an authority to open or apply a draft.
const LAST_PENDING=SCHEMA+'.last_pending';
const LIMIT=1_000_000;
function checked(value){
  if(!value||typeof value.project_id!=='string'||!value.project_id||value.project_id.length>1024
    ||typeof value.profile!=='string'||!Array.isArray(value.files)||!value.files.length||value.files.length>256)
    throw new Error('E_EDIT_RECOVERY_FIELDS: 임시 편집 자료의 필드가 다릅니다.');
  const paths=new Set();
  const files=value.files.map(file=>{
    if(typeof file.path!=='string'||!file.path||paths.has(file.path)||typeof file.text!=='string')
      throw new Error('E_EDIT_RECOVERY_FILES: 임시 파일 목록이 다릅니다.');
    paths.add(file.path);return {path:file.path,text:file.text};
  });
  const result={project_id:value.project_id,profile:value.profile,files:files.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0)};
  if(JSON.stringify(result).length>LIMIT)throw new Error('E_EDIT_RECOVERY_LIMIT: 임시 보관 한도입니다. 원문을 파일로 내보내세요.');
  return result;
}
const token=value=>JSON.stringify(checked(value));
export function createEditRecoveryGuard({storage,capture,applyDraft,present}){
  const saveTokens=new Map();let fault='',dismissed='',offered=false;
  const current=()=>{const value=capture();return value?checked(value):null;};
  const key=id=>PREFIX+encodeURIComponent(id);
  function read(id){
    const text=storage.getItem(key(id));if(text===null)return null;
    if(text.length>LIMIT+4096)throw new Error('E_EDIT_RECOVERY_LIMIT: 임시 자료 크기를 확인할 수 없습니다.');
    const parsed=JSON.parse(text);
    if(parsed.schema!==SCHEMA||typeof parsed.pending!=='boolean'||(parsed.error!==undefined&&(typeof parsed.error!=='string'||parsed.error.length>600)))throw new Error('E_EDIT_RECOVERY_SCHEMA: 지원하지 않는 임시 자료입니다.');
    const data=checked(parsed.source);
    if(data.project_id!==id)throw new Error('E_EDIT_RECOVERY_PROJECT: 다른 작품의 임시 자료입니다.');
    return {...parsed,source:data};
  }
  function showFailure(error){offered=true;present({visible:true,canRestore:false,message:`${String(error?.message??error)} 임시 보관에 실패했습니다. 편집기를 닫지 말고 원문을 내보내세요.`});}
  function offer(){
    try{
      const source=current();if(!source){present({visible:Boolean(fault),canRestore:false,message:fault});return;}
      const record=read(source.project_id);const pending=record?.pending===true;
      if(!pending&&!fault) {
        const otherId=storage.getItem(LAST_PENDING);
        if(otherId&&otherId!==source.project_id) {
          const other=read(otherId),fingerprint=other?.pending?token(other.source):'';
          if(fingerprint&&dismissed!==fingerprint) {
            offered=true;present({visible:true,canRestore:false,canOpen:true,
              message:'다른 작품에 저장 안 된 편집이 남아 있습니다. 저장 자료를 다시 연 뒤 편집 복구를 누르세요. 원문이나 실행 상태를 자동으로 바꾸지 않았습니다.'});return;
          }
        }
      }
      const fingerprint=pending?token(record.source):'';
      const compatible=pending&&source.profile===record.source.profile
        &&JSON.stringify(source.files.map(f=>f.path))===JSON.stringify(record.source.files.map(f=>f.path));
      offered=Boolean(fault)||(pending&&dismissed!==fingerprint);
      present({visible:offered,canRestore:Boolean(compatible),
        message:[fault||(pending?record.error:''),pending?'저장 안 된 편집이 있습니다. 정식 저장본·실행 상태는 바꾸지 않았습니다.':'',
          pending&&!compatible?'현재 작품의 파일 구조와 달라 자동 복구하지 않습니다.':''].filter(Boolean).join(' ')});
    }catch(error){showFailure(error);}
  }
  function saveEditor(activePath,text){
    try{
      const source=current();if(!source)return;
      const prior=read(source.project_id);
      // Preserve an earlier edited file even if a later UI handler threw
      // before it could update the in-memory file map.
      if(prior?.pending&&prior.source.profile===source.profile
          &&JSON.stringify(prior.source.files.map(f=>f.path))===JSON.stringify(source.files.map(f=>f.path)))source.files=prior.source.files.map(f=>({...f}));
      const file=source.files.find(f=>f.path===activePath);
      if(!file)throw new Error('E_EDIT_RECOVERY_ACTIVE_FILE: 현재 파일이 작품에 없습니다.');
      file.text=String(text);const checkedSource=checked(source);
      storage.setItem(key(source.project_id),JSON.stringify({schema:SCHEMA,pending:true,source:checkedSource}));
      storage.setItem(LAST_PENDING,source.project_id);
      dismissed='';if(fault||offered)offer();
    }catch(error){showFailure(error);}
  }
  return {
    saveEditor,offer,
    fault(error){
      fault=`화면 오류: ${String(error?.message??error??'알 수 없는 오류').slice(0,500)}. 정식 저장본은 이 임시 복구 기능으로 바꾸지 않습니다.`;
      try{const source=current(),record=source&&read(source.project_id);if(record?.pending)storage.setItem(key(source.project_id),JSON.stringify({...record,error:fault}));}catch{}
      dismissed='';offer();
    },
    dismiss(){try{const source=current(),record=(source&&read(source.project_id))??read(storage.getItem(LAST_PENDING));dismissed=record?.pending?token(record.source):'';}catch{}fault='';offered=false;present({visible:false,canRestore:false,message:''});},
    beginSave(generation){try{const source=current();if(source)saveTokens.set(generation,token(source));}catch(error){showFailure(error);}},
    saved(generation){
      try{
        const expected=saveTokens.get(generation);saveTokens.delete(generation);
        const source=current();if(!expected||!source)return;
        if(token(source)!==expected){offer();return;}
        const record=read(source.project_id);
        if(record?.pending&&token(record.source)===expected){storage.setItem(key(source.project_id),JSON.stringify({...record,pending:false}));offer();}
      }catch(error){showFailure(error);}
    },
    failed(generation){saveTokens.delete(generation);offer();},
    async restore(){
      try{
        const source=current(),record=source&&read(source.project_id);
        if(!record?.pending)throw new Error('E_EDIT_RECOVERY_EMPTY: 복구할 편집이 없습니다.');
        if(source.profile!==record.source.profile||JSON.stringify(source.files.map(f=>f.path))!==JSON.stringify(record.source.files.map(f=>f.path)))
          throw new Error('E_EDIT_RECOVERY_STRUCTURE: 현재 파일 구조와 다른 편집은 적용하지 않습니다.');
        await applyDraft(record.source);fault='';dismissed='';offer();return true;
      }catch(error){present({visible:true,canRestore:false,message:String(error?.message??error)});return false;}
    },
  };
}
