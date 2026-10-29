"""Logical mode03 stays on its existing INITIAL custody producer/consumer."""
import argparse, hashlib, json, os, sqlite3, subprocess, time
from pathlib import Path
from beta2_native_kill_runner import ROOT, STAGES, sha, write

def main():
 p=argparse.ArgumentParser();p.add_argument('--bins',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--one',action='store_true')
 args=p.parse_args();out=args.out.resolve();out.mkdir(parents=True,exist_ok=False)
 example=ROOT/'solutions/seamgrim_ui_mvp/samples/beta1_logical_tetris'
 artifact=example/'authored-project.json';initial=example/'initial-input.json';next_input=example/'next-input.json'
 first=json.loads(initial.read_text('utf8'));tail=json.loads(next_input.read_text('utf8'))
 assert first['binding']==tail['binding']
 full=dict(first);full['snapshots']=first['snapshots']+tail['snapshots'];continuous=out/'continuous-input.json';write(continuous,full)
 def run(exe,cmd,directory,label):
  result=subprocess.run([str(exe),*map(str,cmd)],cwd=ROOT,capture_output=True,timeout=45)
  (directory/(label+'.stdout')).write_bytes(result.stdout);(directory/(label+'.stderr')).write_bytes(result.stderr);return result
 rows=[];matrix={'claim':'LOGICAL_FIXED_SEED_INITIAL_CUSTODY_CLI3_FIRST_SAVE_KILLS_ONLY','profile':'b2-provisional-1',
  'rows':rows,'source_sha256':sha(artifact.read_bytes()),'initial_input_sha256':sha(initial.read_bytes()),'next_input_sha256':sha(next_input.read_bytes()),
  'power_loss':'NOT_RUN','chrome':'NOT_RUN','subsequent_transaction':'NOT_RUN','all_beta2':'NOT_COMPLETE'}
 for name in (['ddonilang.exe'] if args.one else ['ddonilang.exe','ddonirang.exe','또니랑.exe']):
  exe=args.bins/name;baseline=out/(name+'-continuous');baseline.mkdir();baseline_store=baseline/'store';baseline_store.mkdir()
  expected=run(exe,['작품자료','논리기록',artifact,'--입력',continuous,'--부모','처음','--판','02','--저장소',baseline_store,'--저장소-초기화'],baseline,'continuous')
  assert expected.returncode==0,expected.stderr.decode('utf8',errors='replace')
  reference=json.loads(expected.stdout)['summary'];assert reference['state_hash']=='blake3:c7c9359477b58adc903fe72ad9b1c9105b5da8158db169842ef5a36d4468202d'
  for stage in (STAGES[:1] if args.one else STAGES):
   case=out/(name+'-'+stage);case.mkdir();store=case/'store';store.mkdir();recovery=case/'recovery'
   row={'binary':name,'binary_sha256':sha(exe.read_bytes()),'stage':stage,'status':'NOT_RUN'};rows.append(row);child=None
   try:
    env=os.environ.copy();env.update(DDN_B2_CRASH_STAGE=stage,DDN_B2_CRASH_MARKER=str(case/'reached'))
    command=[str(exe),'작품자료','내구기록',str(artifact),'--입력',str(initial),'--판','02','--저장소',str(store),'--저장소-초기화','--복구자료',str(recovery)]
    with (case/'kill.stdout').open('wb') as stdout,(case/'kill.stderr').open('wb') as stderr:
     child=subprocess.Popen(command,cwd=ROOT,env=env,stdout=stdout,stderr=stderr);deadline=time.monotonic()+25
     while not (case/'reached').exists():
      assert child.poll() is None,'exit before rendezvous: '+(case/'kill.stderr').read_text('utf8',errors='replace')
      assert time.monotonic()<deadline,'rendezvous timeout';time.sleep(.02)
     assert (case/'reached').read_text('utf8')==stage
     row['pid']=child.pid;child.kill();assert child.wait(timeout=5)!=0;row['actual_kill']=True
    assert (case/'kill.stdout').read_bytes()==b''
    with sqlite3.connect('file:'+(store/'vc04_snapshot_v2.sqlite3').as_posix()+'?mode=ro',uri=True) as db:
     assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
     head=db.execute('SELECT history_revision_identity FROM vc04_project_heads').fetchone()
     assert (head is not None)==(stage in STAGES[3:]);row['head']=head[0].hex() if head else None
    token=(recovery/'token.hex').read_text('utf8');locator=(recovery/'grant.txt').read_text('utf8')
    reopened=run(exe,['작품자료','복구',recovery/'saved.json','--판','02','--저장소',store,'--허가',locator,
       '--요청',recovery/'request.bin','--토큰',token,'--입력',next_input],case,'cold')
    if stage=='after_receipt':
     assert reopened.returncode==0,reopened.stderr.decode('utf8',errors='replace')
     actual=json.loads(reopened.stdout);assert actual['state_hash']==reference['state_hash']
     assert actual['madi_count']==reference['madi_count']
     row.update(outcome='NORMAL_COMMITTED_WORLD_CONTINUE_EQUALS_7',state_hash=actual['state_hash'])
    else:
     assert reopened.returncode!=0 and not reopened.stdout,'partial success published'
     error=reopened.stderr.decode('utf8',errors='replace');assert 'RecoveryRequired' in error,error
     row.update(outcome='RECOVERY_REQUIRED',diagnostic=error)
    row['status']='PASS'
   except Exception as error:row.update(status='FAIL',error=str(error))
   finally:
    if child is not None and child.poll() is None:child.kill();child.wait(timeout=5)
    write(out/'B2_KILL_MATRIX.partial.json',matrix)
   print(name,stage,row['status'],flush=True)
 matrix['pass']=sum(row['status']=='PASS' for row in rows);matrix['fail']=sum(row['status']=='FAIL' for row in rows)
 write(out/'B2_KILL_MATRIX.partial.json',matrix);return 1 if matrix['fail'] else 0

if __name__=='__main__':raise SystemExit(main())
