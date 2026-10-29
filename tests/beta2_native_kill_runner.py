"""WO22 actual CLI3 kills. No source lowering, authority fabrication or DB edits."""
import argparse, hashlib, json, os, sqlite3, subprocess, time
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
STAGES=['before_prepared','after_prepared','before_cas','after_cas','after_confirmation','after_receipt']
def sha(b):return hashlib.sha256(b).hexdigest()
def write(path,value):path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf8')

def main():
 p=argparse.ArgumentParser();p.add_argument('--bins',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
 p.add_argument('--one',action='store_true');p.add_argument('--fixtures',nargs='+',choices=['beta1_logical_counter','beta1_logical_tetris','subject'])
 p.add_argument('--stages',nargs='+',choices=STAGES)
 p.add_argument('--subject-authored',type=Path,help='기존 제품이 생성한 같은 임자 작품 bytes 재사용')
 p.add_argument('--aliases',nargs='+',choices=['ddonilang.exe','ddonirang.exe','또니랑.exe'],help='이미 끝난 별칭 관문을 반복하지 않고 남은 별칭만 실행')
 args=p.parse_args();out=args.out.resolve();out.mkdir(exist_ok=False,parents=True)
 fixtures={name:(ROOT/'solutions/seamgrim_ui_mvp/samples'/name/'authored-project.json').read_bytes()
           for name in ['beta1_logical_counter','beta1_logical_tetris']}
 subject='명찰:이름씨 := { 이름:글. }. 병사:임자 := { 이름:글. }. 채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름="철수")인 병사. (표적 가리키기) 보여주기. }. }.'
 request={'source_profile':'v1-core-v25','module_name':'','source_snapshot':{'root_path':'root.ddn','entry_path':'main.ddn','files':[
  {'path':'root.ddn','text':'(글판="v25", 모듬=차림[(이름=앱, 머리="main.ddn", 원문="main.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.'},
  {'path':'main.ddn','text':subject}]}}
 create=out/'subject-create.json';write(create,request)
 def execute(exe,cmd,case,tag):
  result=subprocess.run([str(exe),*map(str,cmd)],cwd=ROOT,capture_output=True,timeout=40)
  (case/(tag+'.stdout')).write_bytes(result.stdout);(case/(tag+'.stderr')).write_bytes(result.stderr)
  return result
 if args.subject_authored:
  fixtures['subject']=args.subject_authored.read_bytes()
  assert json.loads(fixtures['subject'])['source_snapshot']==request['source_snapshot'], '기존 시험과 같은 원문이어야 합니다'
 else:
  first=args.bins/'ddonilang.exe';created=execute(first,['작품자료','만들기',create],out,'create')
  assert created.returncode==0,created.stderr.decode('utf8',errors='replace')
  fixtures['subject']=created.stdout
 (out/'subject-authored.json').write_bytes(fixtures['subject'])
 all_fixtures=list(fixtures)
 if args.fixtures:fixtures={k:v for k,v in fixtures.items() if k in args.fixtures}
 rows=[];manifest={'claim':'CLI3_SOURCE_PARENT_DEVELOPMENT_PROBE_NOT_POWER_LOSS','profile':'b2-provisional-1',
  'rows':rows,'chrome':'NOT_RUN','corruption':'NOT_RUN','all_beta2':'NOT_COMPLETE',
  'unrun_required_fixtures':[name for name in all_fixtures if name not in fixtures],
  'input_path':'SOURCE_FIRST_GENERAL_NO_EXTERNAL_INPUT_NOT_LOGICAL_KEYBOARD'}
 manifest['subject_input']={'reused_from':str(args.subject_authored) if args.subject_authored else None,
  'sha256':sha(fixtures['subject']) if 'subject' in fixtures else sha((out/'subject-authored.json').read_bytes())}
 stopped=False
 for executable in (['ddonilang.exe'] if args.one else (args.aliases or ['ddonilang.exe','ddonirang.exe','또니랑.exe'])):
  if stopped:break
  exe=args.bins/executable
  for fixture,raw in fixtures.items():
   if stopped:break
   stages=STAGES[:1] if args.one else (args.stages or STAGES)
   for stage in stages:
    case=out/(executable+'-'+fixture+'-'+stage);case.mkdir();artifact=case/'authored.json';artifact.write_bytes(raw)
    row={'binary':executable,'binary_sha256':sha(exe.read_bytes()),'fixture':fixture,'source_sha256':sha(raw),'stage':stage,'status':'NOT_RUN'}
    rows.append(row)
    child=None
    try:
     store=case/'store';store.mkdir()
     saved=execute(exe,['작품자료','원문저장',artifact,'--판','01','--저장소',store],case,'source-save')
     assert saved.returncode==0,saved.stderr.decode('utf8',errors='replace')
     source_report=case/'source-report.json';source_report.write_bytes(saved.stdout);locator=json.loads(saved.stdout)['grant_locator']
     dbpath=store/'vc04_snapshot_v2.sqlite3'
     with sqlite3.connect('file:'+dbpath.as_posix()+'?mode=ro',uri=True) as db:
      source=sha(db.execute('SELECT project_payload_bytes FROM vc04_prepared_payloads WHERE payload_kind=0').fetchone()[0])
     env=os.environ.copy();env.update(DDN_B2_CRASH_STAGE=stage,DDN_B2_CRASH_MARKER=str(case/'reached'),DDN_B2_CRASH_CAPTURE=str(case/'candidate.json'))
     with (case/'kill.stdout').open('wb') as stdout,(case/'kill.stderr').open('wb') as stderr:
      child=subprocess.Popen([str(exe),'작품자료','첫실행',str(source_report),'--판','02','--저장소',str(store),'--마디수','2'],cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
      deadline=time.monotonic()+25
      while not (case/'reached').exists():
       assert child.poll() is None,'producer exited before rendezvous: '+(case/'kill.stderr').read_text('utf8',errors='replace')
       assert time.monotonic()<deadline,'rendezvous timeout'
       time.sleep(.02)
      assert (case/'reached').read_text('utf8')==stage
      row['pid']=child.pid;child.kill();code=child.wait(timeout=5);assert code!=0
      row['actual_kill']=True
     assert (case/'kill.stdout').read_bytes()==b'', 'producer published success before kill'
     capture=json.loads((case/'candidate.json').read_text('utf8'));resume=case/'resume.json'
     resume.write_text(capture['artifact'],encoding='utf8')
     with sqlite3.connect('file:'+dbpath.as_posix()+'?mode=ro',uri=True) as db:
      assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
      assert sha(db.execute('SELECT project_payload_bytes FROM vc04_prepared_payloads WHERE payload_kind=0').fetchone()[0])==source
      head=db.execute('SELECT history_revision_identity FROM vc04_project_heads').fetchone()[0].hex()
      row['head']=head
      assert head==('02' if stage in STAGES[3:] else '01')
     reopened=execute(exe,['작품자료','원문이어실행',resume,'--부모','01','--판','02','--저장소',store,'--허가',locator,'--마디수','1'],case,'cold')
     if stage=='after_receipt':
      assert reopened.returncode==0,reopened.stderr.decode('utf8',errors='replace')
      actual=json.loads(reopened.stdout);assert actual['state_hash']==capture['continuous_reference']['state_hash']
      # Continued summaries expose next_tick_id; a fresh continuous run uses
      # ticks_run. Both start at tick0 in this explicit first-world fixture.
      assert actual['next_tick_id']==capture['continuous_reference']['ticks_run']
      row.update(outcome='NORMAL_COMMITTED_WORLD',state_hash=actual['state_hash'])
     else:
      assert reopened.returncode!=0 and not reopened.stdout,'partial success on cold admission'
      error=reopened.stderr.decode('utf8',errors='replace');assert 'RecoveryRequired' in error,error
      row.update(outcome='RECOVERY_REQUIRED',diagnostic=error)
     row.update(status='PASS',source_preserved=True)
    except Exception as error:row.update(status='FAIL',error=str(error))
    finally:
     if child is not None and child.poll() is None:child.kill();child.wait(timeout=5)
     write(out/'B2_KILL_MATRIX.partial.json',manifest)
    print(executable,fixture,stage,row['status'],flush=True)
    if len(rows)>=2 and all(row['status']=='FAIL' for row in rows[-2:]):
     manifest['stopped_after_two_failures']=True;stopped=True
     print('STOPPED_AFTER_TWO_FAILURES: inspect raw cases before retry',flush=True);break
 manifest['pass']=sum(row['status']=='PASS' for row in rows);manifest['fail']=sum(row['status']=='FAIL' for row in rows)
 write(out/'B2_KILL_MATRIX.partial.json',manifest)
 return 1 if manifest['fail'] else 0

if __name__=='__main__':raise SystemExit(main())
