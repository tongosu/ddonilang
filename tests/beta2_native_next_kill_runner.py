"""Actual CLI next save, kill, cold reopen; report B2-04 divergence honestly."""
import argparse,hashlib,json,os,sqlite3,subprocess,time
from pathlib import Path
STAGES=['before_prepared','after_prepared','before_cas','after_cas','after_confirmation','after_receipt']
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('--bins',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--subject',type=Path);p.add_argument('--one',action='store_true');p.add_argument('--stages',nargs='+',choices=STAGES)
    args=p.parse_args();args.out.mkdir(parents=True,exist_ok=False);root=Path(__file__).resolve().parents[1]
    raw=(args.subject or root/'solutions/seamgrim_ui_mvp/samples/beta1_logical_counter/authored-project.json').read_bytes()
    env={k:v for k,v in os.environ.items() if not k.startswith('DDN_B2_CRASH_')};rows=[]
    def execute(exe,case,name,command):
        result=subprocess.run([str(exe),*map(str,command)],cwd=root,env=env,capture_output=True,timeout=90)
        (case/f'{name}.stdout').write_bytes(result.stdout);(case/f'{name}.stderr').write_bytes(result.stderr);return result
    for binary in (['ddonilang.exe'] if args.one else ['ddonilang.exe','ddonirang.exe','또니랑.exe']):
        exe=(args.bins/binary).resolve()
        for stage in args.stages or STAGES:
            case=args.out/(binary+'-'+stage);case.mkdir();store=case/'store';store.mkdir()
            artifact=case/'authored.json';artifact.write_bytes(raw)
            row={'binary':binary,'binary_sha256':sha(exe.read_bytes()),'stage':stage,'status':'NOT_RUN'};rows.append(row);child=None
            try:
                saved=execute(exe,case,'source',['작품자료','원문저장',artifact,'--판','01','--저장소',store]);assert saved.returncode==0,saved.stderr.decode('utf8',errors='replace')
                locator=json.loads(saved.stdout)['grant_locator']
                old=case/'old.json'
                first=execute(exe,case,'first',['작품자료','첫실행',case/'source.stdout','--판','02','--저장소',store,'--마디수','2','--저장파일',old]);assert first.returncode==0,first.stderr.decode('utf8',errors='replace')
                first_report=json.loads(first.stdout);assert old.read_text(encoding='utf8')==first_report['saved_artifact_json_utf8']
                def resume(path,parent,revision,ticks):return ['작품자료','원문이어실행',path,'--부모',parent,'--판',revision,'--저장소',store,'--허가',locator,'--마디수',str(ticks)]
                # Actual verified old world followed continuously by two ticks.
                control=execute(exe,case,'continuous',resume(old,'01','02',2));assert control.returncode==0,control.stderr.decode('utf8',errors='replace')
                reference=json.loads(control.stdout)
                database=store/'vc04_snapshot_v2.sqlite3'
                with sqlite3.connect(database) as db:
                    prior=db.execute('SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE payload_kind=1').fetchone()[0]
                candidate=case/'next.json'
                probe={**env,'DDN_B2_CRASH_STAGE':stage,'DDN_B2_CRASH_MARKER':str((case/'reached').resolve())}
                with (case/'producer.stdout').open('wb') as stdout,(case/'producer.stderr').open('wb') as stderr:
                    child=subprocess.Popen([str(exe),*map(str,resume(old,'01','02',1)),'--저장판','03','--저장파일',str(candidate)],cwd=root,env=probe,stdout=stdout,stderr=stderr)
                    deadline=time.monotonic()+90
                    while not (case/'reached').exists():
                        assert child.poll() is None,'producer terminated before rendezvous'
                        assert time.monotonic()<deadline,'actual rendezvous observation expired'
                        time.sleep(.03)
                    assert (case/'reached').read_text()==stage
                    row['killed_pid']=child.pid;child.kill();assert child.wait(timeout=10)!=0;child=None
                assert not (case/'producer.stdout').read_bytes(),'success published before kill'
                assert candidate.is_file(),'the real CLI did not preserve its explicit restore material'
                assert not (case/'candidate.json').exists(),'probe observation must not supply restore material'
                row['restore_artifact_producer']='PRODUCT_CLI_EXPLICIT_FILE_NOT_PROBE'
                with sqlite3.connect(database) as db:
                    head=db.execute('SELECT history_revision_identity FROM vc04_project_heads').fetchone()[0].hex()
                    assert head==('02' if stage in STAGES[:3] else '03')
                    assert prior in [r[0] for r in db.execute('SELECT canonical_snapshot_v2_bytes FROM vc04_prepared_payloads WHERE payload_kind=1')]
                row.update(head=head,prior_snapshot_bytes_preserved=True)
                old_result=execute(exe,case,'cold_old',resume(old,'01','02',1))
                next_result=execute(exe,case,'cold_next',resume(candidate,'02','03',1))
                if stage=='after_receipt':
                    assert next_result.returncode==0,next_result.stderr.decode('utf8',errors='replace')
                    actual=json.loads(next_result.stdout);assert actual['state_hash']==reference['state_hash'];assert actual['next_tick_id']==reference['next_tick_id']
                    row.update(next_outcome='NORMAL_VERIFIED_WORLD',state_hash=actual['state_hash'])
                else:
                    assert next_result.returncode!=0 and not next_result.stdout,'partial candidate success'
                    row.update(next_outcome='REJECTED_NO_WORLD',next_diagnostic=next_result.stderr.decode('utf8',errors='replace'))
                row['old_outcome']='NORMAL_VERIFIED_WORLD' if old_result.returncode==0 else 'REJECTED_NO_WORLD'
                if old_result.returncode!=0:assert not old_result.stdout;row['old_diagnostic']=old_result.stderr.decode('utf8',errors='replace')
                if stage in STAGES[:3]:row['b2_04_prior_normal_match']=old_result.returncode==0
                row['status']='OBSERVATION_VERIFIED'
            except Exception as error:row.update(status='FAIL',error=str(error))
            finally:
                if child is not None and child.poll() is None:child.kill();child.wait(timeout=10)
                result={'claim':'CLI_SECOND_TRANSACTION_OBSERVATIONS_NOT_B2_04_PASS','source_sha256':sha(raw),'rows':rows,
                    'observations_verified':sum(r['status']=='OBSERVATION_VERIFIED' for r in rows),'fail':sum(r['status']=='FAIL' for r in rows),
                    'b2_04_deviations':sum(r.get('b2_04_prior_normal_match') is False for r in rows)}
                (args.out/'B2_NEXT_CLI_MATRIX.partial.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
            print(binary,stage,row['status'],row.get('error',''),flush=True)
            if row['status']=='FAIL':raise SystemExit(1)
if __name__=='__main__':main()
