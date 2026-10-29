"""CLI3 real process exit/cold authoring, no harness source lowering."""
from pathlib import Path
import os,json,subprocess,hashlib,tempfile
stage=Path(os.environ['DDN_A_STAGE']);out=Path(os.environ['DDN_A_OUT']);out.mkdir()
raw=(stage/'solutions/seamgrim_ui_mvp/samples/beta1_logical_counter/authored-project.json').read_text(encoding='utf8')
reports=[]
for alias in ['ddonilang','ddonirang','또니랑']:
 case=out/alias;case.mkdir();counter=0
 binary=stage/'bin'/(alias+'.exe');env=os.environ.copy()
 reused=Path(os.environ['DDN_A_REUSE_CLI_R1'])/alias if alias=='ddonilang' and os.environ.get('DDN_A_REUSE_CLI_R1') else None
 env['LOCALAPPDATA']=str(reused/'native-local') if reused else tempfile.mkdtemp(prefix='ddn-authoring-cli-')
 def command(args,request,ok=True):
  global counter
  counter+=1;file=case/f'{counter:02}-request.json';file.write_text(json.dumps(request,ensure_ascii=False),encoding='utf8')
  result=subprocess.run([str(binary),'작품자료',*args,str(file)],cwd=stage,env=env,capture_output=True,timeout=180)
  (case/f'{counter:02}.stdout').write_bytes(result.stdout);(case/f'{counter:02}.stderr').write_bytes(result.stderr)
  if ok:assert result.returncode==0,(args,result.stderr.decode('utf8',errors='replace'))
  else:assert result.returncode!=0
  return json.loads(result.stdout) if ok else result.stderr.decode('utf8',errors='replace')
 def op(name,artifact,**fields):return command(['원문전이'],dict(fields,operation=name,authored_project_json_utf8=artifact))
 def event(amount,summary):return {'amount':amount,'summary':summary,'old_artifact_sha256':hashlib.sha256(first['saved_artifact_json_utf8'].encode()).hexdigest()}
 if reused:
  first=json.loads((reused/'03.stdout').read_text(encoding='utf8'));expected=json.loads((reused/'12.stdout').read_text(encoding='utf8'))['summary']
  committed=json.loads((reused/'13.stdout').read_text(encoding='utf8'));cold=json.loads((reused/'14.stdout').read_text(encoding='utf8'))
  assert {k:v for k,v in cold['summary'].items() if k!='frames'}=={k:v for k,v in expected.items() if k!='frames'}
  previous=cold['authored_project_json_utf8'];selected=committed['authoring_selection'];counter=14;events=[event(4,expected)];amounts=[5]
 else:
  command(['열기'],json.loads(raw));initial=(case/f'{counter:02}.stdout').read_text(encoding='utf8').rstrip('\r\n')
  op('save_source',initial);first=op('connect',initial);selected=first['authoring_selection'];previous=initial;events=[];amounts=[4,5]
 for amount in amounts:
  snapshot=json.loads(previous)['source_snapshot']
  for file in snapshot['files']:
   if file['path']=='lib/힘.ddn':file['text']=file['text'].replace(f'값:수 := {amount-1}.',f'값:수 := {amount}.')
  changed=command(['갱신'],{'original_json':previous,'source_snapshot':snapshot})
  edited=json.dumps(changed,ensure_ascii=False,separators=(',',':'))
  # Rust's canonical open output, not Python re-serialization, supplies raw.
  opened=command(['열기'],changed)
  # Authored CLI returns its exact canonical bytes; preserve those bytes.
  edited=(case/f'{counter:02}.stdout').read_text(encoding='utf8').rstrip('\r\n')
  saved=op('save_source',edited,previous_authored_project_json_utf8=previous,authoring_selection=selected)
  assert saved['requires_new_run']
  ready=op('load',previous);assert ready['authored_project_json_utf8']==edited
  unchanged=op('save_source',edited);assert unchanged==saved
  command(['원문전이'],{'operation':'connect','authored_project_json_utf8':edited},ok=False)
  fresh=op('new_run',edited,authoring_selection=saved['authoring_selection'],seed='0');assert fresh['first_frame_pending']
  command(['원문전이'],{'operation':'new_run','authored_project_json_utf8':edited,'authoring_selection':saved['authoring_selection'],'seed':'0'},ok=False)
  next_result=op('run_to',edited,next_tick='3');expected=next_result['summary']
  assert expected['output_display']['texts']==[str(amount*i) for i in range(1,4)]
  committed=op('save_world',edited,next_tick='3',state_hash=expected['state_hash']);selected=committed['authoring_selection']
  cold=op('load',edited)
  # Per-call `frames` is the newly executed suffix, not stored world state.
  # Cold read executes zero ticks, so compare every retained summary field.
  assert {k:v for k,v in cold['summary'].items() if k!='frames'}=={k:v for k,v in expected.items() if k!='frames'}
  connect=op('connect',edited);assert not connect['first_frame_pending'];assert connect['saved_artifact_json_utf8']==cold['saved_artifact_json_utf8']
  events.append(event(amount,expected))
  previous=edited
 reports.append({'alias':alias,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'events':events,'processes':counter,'native_local':env['LOCALAPPDATA'],'reused_raw':str(reused) if reused else None})
def observable(row):return [(e['summary']['state_hash'],e['summary']['output_display'],e['summary']['resources']) for e in row['events']]
assert all(observable(r)==observable(reports[0]) for r in reports[1:])
record={'verdict':'PASS_CLI3_ACTUAL_PROCESS_AUTHORING_TWO_CYCLES','same_raw':True,'rows':reports}
(out/'RESULT.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(json.dumps({'verdict':record['verdict'],'out':str(out),'processes':[r['processes'] for r in reports]}))
