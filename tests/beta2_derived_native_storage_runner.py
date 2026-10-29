"""Actual CLI3 source save/cold/resave. No evaluator, lowering or fake frame."""
import argparse,hashlib,json,os,subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
SHA=lambda data:hashlib.sha256(data).hexdigest()
HEADER='(글판="v25", 모듬=차림[(이름=앱, 머리="main.ddn", 원문="main.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.'
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--bins',type=Path,required=True);parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--one',action='store_true')
    args=parser.parse_args();args.out.mkdir(parents=True,exist_ok=False)
    rows=[];env={k:v for k,v in os.environ.items() if not k.startswith('DDN_B2_CRASH_')}
    for alias in ['ddonilang','ddonirang','또니랑']:
        exe=args.bins/(alias+'.exe')
        for kind,expression in [('scalar','원천 * 2.0'),('fraction_tuple','(원천, 1 / 3)')]:
            case=args.out/(alias+'-'+kind);case.mkdir();store=case/'store';store.mkdir()
            text=f'채비 {{ 원천:셈수 := 1.0. }}. 결과 <<- {expression}. 실행:움직씨 := {{ (매마디)마다 {{ 원천 <- 원천 + 1.0. 원천 보여주기. }}. }}.'
            request={'source_profile':'v1-core-v25','module_name':'','source_snapshot':{'root_path':'root.ddn','entry_path':'main.ddn',
                'files':[{'path':'root.ddn','text':HEADER},{'path':'main.ddn','text':text}]}}
            create=case/'create.json';create.write_text(json.dumps(request,ensure_ascii=False),encoding='utf8')
            def run(tag,command):
                result=subprocess.run([str(exe),*map(str,command)],cwd=ROOT,env=env,capture_output=True,timeout=120)
                (case/(tag+'.stdout')).write_bytes(result.stdout);(case/(tag+'.stderr')).write_bytes(result.stderr)
                assert result.returncode==0,result.stderr.decode('utf8',errors='replace')
                return result.stdout,json.loads(result.stdout)
            row={'alias':alias,'case':kind,'binary_sha256':SHA(exe.read_bytes()),'raw_main_sha256':SHA(text.encode()),'status':'NOT_RUN'};rows.append(row)
            try:
                raw,_=run('create',['작품자료','만들기',create]);artifact=case/'authored-project.json';artifact.write_bytes(raw)
                _,baseline=run('continuous7',['작품자료','실행',artifact,'--마디수','7'])
                _,baseline8=run('continuous8',['작품자료','실행',artifact,'--마디수','8'])
                raw,source=run('source',['작품자료','원문저장',artifact,'--판','01','--저장소',store]);source_report=case/'source-report.json';source_report.write_bytes(raw)
                locator=source['grant_locator'];saved=case/'saved3.json'
                _,first=run('save3',['작품자료','첫실행',source_report,'--판','02','--저장소',store,'--마디수','3','--저장파일',saved])
                assert saved.read_text('utf8')==first['saved_artifact_json_utf8']
                def resume(path,parent,revision,ticks):return ['작품자료','원문이어실행',path,'--부모',parent,'--판',revision,'--저장소',store,'--허가',locator,'--마디수',str(ticks)]
                _,cold=run('cold4',resume(saved,'01','02',4))
                for key in ['state_hash','resources','output_display']:
                    assert cold[key]==baseline[key],key+' differs after cold continuation'
                next_saved=case/'saved7.json'
                _,resaved=run('resave4',resume(saved,'01','02',4)+['--저장판','03','--저장파일',next_saved])
                assert resaved['continued_state_saved'] is True
                resaved=resaved['summary']
                for key in ['state_hash','resources','output_display']:assert resaved[key]==cold[key],key+' differs at resave'
                # The existing CLI explicitly requires a positive madi count.
                # Reopen the seven-step resave and advance one real step; never
                # weaken that contract to make a zero-step observer pass.
                _,again=run('cold_again1',resume(next_saved,'02','03',1))
                for key in ['state_hash','resources','output_display']:assert again[key]==baseline8[key],key+' differs after second cold open + one step'
                row.update(status='PASS',state_hash=cold['state_hash'],cold_resume_3_plus_4_equals_7=True,resaved7_cold1_equals_continuous8=True,
                    resaved_next_state_hash=again['state_hash'],
                    output_equal=True,resources_equal=True,source_artifact_sha256=SHA(artifact.read_bytes()))
            except Exception as error:
                row.update(status='FAIL',error=str(error));raise
            finally:
                report={'claim':'DERIVED_NATIVE_CLI3_SAVE_COLD_RESAVE_ONLY_NOT_WASM_OR_BETA2_PASS','rows':rows}
                (args.out/'RESULT.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
            print(alias,kind,row['status'],flush=True)
            if args.one:return
if __name__=='__main__':main()
