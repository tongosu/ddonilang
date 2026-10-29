"""Mutate only fresh owned stores after a successful cold control; no repair."""
import argparse, hashlib, json, os, sqlite3, subprocess, sys
from pathlib import Path

def sha(data):return hashlib.sha256(data).hexdigest()
CASES=['record_bit','request_bit','row_hash_bit','database_truncated','record_missing',
       'record_token_missing','record_token_corrupt','record_token_other_project','custody_other_project']

def fields(raw):
    assert raw[0]==14;cursor=1;values=[]
    for index in range(1,15):
        assert raw[cursor]==index
        size=int.from_bytes(raw[cursor+1:cursor+5],'big');cursor+=5
        values.append(raw[cursor:cursor+size]);assert len(values[-1])==size;cursor+=size
    assert cursor==len(raw);return values

def encode(values):
    return bytes([14])+b''.join(bytes([i])+len(value).to_bytes(4,'big')+value for i,value in enumerate(values,1))

def row_hash(digest,ordinal,previous,request,record):
    # Existing Rust InitialCustodyRowV1::row_hash, not an authority producer.
    return hashlib.sha256(b'DDN-NATIVE-RECOVERY-CUSTODY-ROW-V1\0'+(1).to_bytes(2,'big')+
        digest+bytes([ordinal])+previous+len(request).to_bytes(4,'big')+request+
        len(record).to_bytes(4,'big')+record).digest()

def main():
    sys.stdout.reconfigure(encoding='utf8')
    p=argparse.ArgumentParser();p.add_argument('--bin',type=Path,required=True);p.add_argument('--out',type=Path,required=True)
    p.add_argument('--cases',nargs='+',choices=CASES,default=CASES[:4])
    p.add_argument('--logical',action='store_true',help='실제 논리 INITIAL 생산자와 cold 복구 입구 검사')
    args=p.parse_args();args.out.mkdir(parents=True,exist_ok=False)
    root=Path(__file__).resolve().parents[1]
    sample=root/'solutions/seamgrim_ui_mvp/samples'/('beta1_logical_tetris' if args.logical else 'beta1_logical_counter')
    raw=(sample/'authored-project.json').read_bytes()
    env={key:value for key,value in os.environ.items() if not key.startswith('DDN_B2_CRASH_')}
    rows=[]
    def run(case,label,command):
        result=subprocess.run([str(args.bin.resolve()),*map(str,command)],env=env,cwd=root,capture_output=True,timeout=60)
        (case/f'{label}.stdout').write_bytes(result.stdout);(case/f'{label}.stderr').write_bytes(result.stderr)
        return result
    for mutation in args.cases:
        case=args.out/mutation;case.mkdir();store=case/'store';store.mkdir()
        artifact=case/'authored.json';artifact.write_bytes(raw)
        row={'mutation':mutation,'status':'NOT_RUN'};rows.append(row)
        try:
            if args.logical:
                recovery=case/'recovery'
                saved=run(case,'save',['작품자료','내구기록',artifact,'--입력',sample/'initial-input.json','--판','02','--저장소',store,'--저장소-초기화','--복구자료',recovery])
                assert saved.returncode==0,saved.stderr.decode('utf8',errors='replace')
                command=['작품자료','복구',recovery/'saved.json','--판','02','--저장소',store,'--허가',(recovery/'grant.txt').read_text('utf8'),'--요청',recovery/'request.bin','--토큰',(recovery/'token.hex').read_text('utf8'),'--입력',sample/'next-input.json']
            else:
                saved=run(case,'save',['작품자료','원문저장',artifact,'--판','01','--저장소',store])
                assert saved.returncode==0,saved.stderr.decode('utf8',errors='replace')
                locator=json.loads(saved.stdout)['grant_locator']
                first=run(case,'first',['작품자료','첫실행',case/'save.stdout','--판','02','--저장소',store,'--마디수','2'])
                assert first.returncode==0,first.stderr.decode('utf8',errors='replace')
                report=json.loads(first.stdout);candidate=case/'saved-artifact.json'
                candidate.write_text(report['saved_artifact_json_utf8'],encoding='utf8')
                command=['작품자료','원문이어실행',candidate,'--부모','01','--판','02','--저장소',store,'--허가',locator,'--마디수','1']
            control=run(case,'cold_control',command)
            assert control.returncode==0,control.stderr.decode('utf8',errors='replace')
            row['control_state_hash']=json.loads(control.stdout)['state_hash']
            database=store/'vc04_snapshot_v2.sqlite3'
            with sqlite3.connect(database) as db:
                db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
                # INITIAL and source-parent use distinct admitted schemas.
                # Compare every persisted payload field without adding a
                # source-only column or coercing either schema into the other.
                source_before=db.execute('SELECT * FROM vc04_prepared_payloads ORDER BY transaction_digest').fetchall()
                row['head_before']=db.execute('SELECT history_revision_identity FROM vc04_project_heads').fetchone()[0].hex()
            original=database.read_bytes();(case/'before-mutation.sqlite3').write_bytes(original)
            row['database_before_sha256']=sha(original)
            if mutation=='database_truncated':
                # Owned fixture only; retain full original bytes above. This is
                # simulated truncation, not evidence of physical power loss.
                database.write_bytes(original[:max(100,len(original)//2)])
            elif mutation in ['record_missing','record_token_missing','record_token_corrupt','record_token_other_project','custody_other_project']:
                foreign=None
                if mutation in ['record_token_other_project','custody_other_project']:
                    donor=case/'donor';donor.mkdir();donor_store=donor/'store';donor_store.mkdir()
                    create={'source_profile':'v1-core-v25','module_name':'','source_snapshot':{
                        'root_path':'root.ddn','entry_path':'main.ddn','files':[
                        {'path':'root.ddn','text':'(글판="v25", 모듬=차림[(이름=별도앱, 머리="main.ddn", 원문="main.ddn")], 시작=(모듈=별도앱, 정의=실행)) 얽음.'},
                        {'path':'main.ddn','text':'채비 { 수치:바른수 := 17. }. 실행:움직씨 := { (매마디)마다 { 수치 <- 수치 + 1. 수치 보여주기. }. }.'}]}}
                    create_path=donor/'create.json';create_path.write_text(json.dumps(create,ensure_ascii=False),encoding='utf8')
                    if args.logical:
                        original_artifact=json.loads(raw)
                        create={'source_profile':original_artifact['source_profile'],'module_name':'','source_snapshot':original_artifact['source_snapshot']}
                        create_path.write_text(json.dumps(create,ensure_ascii=False),encoding='utf8')
                    created=run(donor,'create',['작품자료','만들기',create_path]);assert created.returncode==0,created.stderr.decode('utf8',errors='replace')
                    donor_artifact=donor/'authored.json';donor_artifact.write_bytes(created.stdout)
                    assert json.loads(created.stdout)['authored_identity']['logical_dependency_id']!=json.loads(raw)['authored_identity']['logical_dependency_id']
                    if args.logical:
                        donor_world=run(donor,'first',['작품자료','내구기록',donor_artifact,'--입력',sample/'initial-input.json','--판','02','--저장소',donor_store,'--저장소-초기화','--복구자료',donor/'recovery'])
                    else:
                        donor_source=run(donor,'save',['작품자료','원문저장',donor_artifact,'--판','01','--저장소',donor_store]);assert donor_source.returncode==0,donor_source.stderr.decode('utf8',errors='replace')
                        donor_world=run(donor,'first',['작품자료','첫실행',donor/'save.stdout','--판','02','--저장소',donor_store,'--마디수','2'])
                    assert donor_world.returncode==0,donor_world.stderr.decode('utf8',errors='replace')
                    with sqlite3.connect(donor_store/'vc04_snapshot_v2.sqlite3') as donor_db:
                        foreign=[bytes(row[0]) for row in donor_db.execute('SELECT recovery_record_bytes FROM vc04_recovery_custody_v1 ORDER BY journal_ordinal')]
                    assert len(foreign)==3
                with sqlite3.connect(database) as db:
                    records=db.execute('SELECT transaction_digest,journal_ordinal,previous_row_sha256,admitted_request_bytes,recovery_record_bytes,row_sha256 FROM vc04_recovery_custody_v1 ORDER BY journal_ordinal').fetchall()
                    assert len(records)==3
                    if mutation=='record_missing':
                        digest,ordinal,*_=records[0];displaced=bytearray(digest);displaced[-1]^=1
                        # Preserve fixture bytes and the row itself; move its
                        # query key so the actual transaction has no ordinal0.
                        db.execute('UPDATE vc04_recovery_custody_v1 SET transaction_digest=? WHERE transaction_digest=? AND journal_ordinal=?',(bytes(displaced),digest,ordinal))
                    else:
                        previous=bytes(32)
                        for digest,ordinal,old_previous,request,record,old_hash in records:
                            assert row_hash(digest,ordinal,old_previous,request,record)==old_hash,'test hash does not match actual producer'
                            values=fields(record)
                            if mutation=='record_token_missing':values[11]=b''
                            elif mutation=='record_token_corrupt':
                                token=bytearray(values[11]);token[-1]^=1;values[11]=bytes(token)
                            elif mutation=='record_token_other_project':values[11]=fields(foreign[ordinal])[11]
                            elif mutation=='custody_other_project':values=fields(foreign[ordinal])
                            changed=encode(values);digest_row=row_hash(digest,ordinal,previous,request,changed)
                            db.execute('UPDATE vc04_recovery_custody_v1 SET previous_row_sha256=?,recovery_record_bytes=?,row_sha256=? WHERE transaction_digest=? AND journal_ordinal=?',(previous,changed,digest_row,digest,ordinal));previous=digest_row
                        row['outer_row_chain_recomputed']=True
                        row['mutation_layer']='record field12 token' if mutation.startswith('record_token_') else 'actual foreign project recovery records'
                    db.commit()
            else:
                column={'record_bit':'recovery_record_bytes','request_bit':'admitted_request_bytes','row_hash_bit':'row_sha256'}[mutation]
                with sqlite3.connect(database) as db:
                    digest,ordinal,blob=db.execute(f'SELECT transaction_digest,journal_ordinal,{column} FROM vc04_recovery_custody_v1 ORDER BY journal_ordinal LIMIT 1').fetchone()
                    changed=bytearray(blob);changed[-1]^=1
                    db.execute(f'UPDATE vc04_recovery_custody_v1 SET {column}=? WHERE transaction_digest=? AND journal_ordinal=?',(bytes(changed),digest,ordinal));db.commit()
            rejected=run(case,'cold_mutated',command)
            assert rejected.returncode!=0 and not rejected.stdout,'damaged store published success'
            diagnostic=rejected.stderr.decode('utf8',errors='replace');assert diagnostic.strip()
            if mutation!='database_truncated':
                if args.logical:assert diagnostic.startswith('E_NATIVE_CUSTODY: 복구 확인 실패 ('),diagnostic
                else:assert diagnostic.startswith('E_SOURCE_WORLD_COLD_ADMISSION: 복구 필요'),diagnostic
            assert artifact.read_bytes()==raw,'original authored artifact changed'
            row['diagnostic']=diagnostic;row['partial_success_published']=False
            if mutation!='database_truncated':
                with sqlite3.connect(database) as db:
                    assert db.execute('SELECT * FROM vc04_prepared_payloads ORDER BY transaction_digest').fetchall()==source_before
                    assert db.execute('SELECT history_revision_identity FROM vc04_project_heads').fetchone()[0].hex()==row['head_before']
                row['source_and_head_unchanged']=True
            row['status']='PASS'
        except Exception as error:row.update(status='FAIL',error=str(error))
        result={'claim':'OWNED_NATIVE_CUSTODY_MUTATIONS_NOT_POWER_LOSS',
            'producer':'logical-initial' if args.logical else 'general-source-parent',
            'binary_sha256':sha(args.bin.read_bytes()),'source_sha256':sha(raw),'rows':rows,
            'pass':sum(row['status']=='PASS' for row in rows),'fail':sum(row['status']=='FAIL' for row in rows),
            'unrun_mutations':[item for item in CASES if not any(row['mutation']==item for row in rows)],
            'not_covered':['Chrome','physical power loss']+([] if args.logical else ['logical mode03 token transport'])}
        (args.out/'B2_CUSTODY_MUTATIONS.partial.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
        print(mutation,row['status'],row.get('error',''),flush=True)
        if len(rows)>=2 and all(row['status']=='FAIL' for row in rows[-2:]):
            print('STOPPED_AFTER_TWO_FAILURES: inspect raw cases before retry',flush=True);break
    if any(row['status']!='PASS' for row in rows):raise SystemExit(1)
if __name__=='__main__':main()
