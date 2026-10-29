"""Sequential actual Chrome kills; one transaction class, never a Beta2 verdict."""
import argparse, hashlib, json, subprocess
from pathlib import Path

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--stage',type=Path,required=True)
    p.add_argument('--subject',type=Path)
    p.add_argument('--logical',action='store_true',help='Use the real logical-input Native Workbench flow for Tetris')
    p.add_argument('--out',type=Path,required=True)
    p.add_argument('--transaction',choices=['first','next'],default='first')
    p.add_argument('--fixtures',nargs='+',choices=['counter','tetris','subject'],default=['counter','tetris','subject'])
    p.add_argument('--stages',nargs='+',choices=['before_prepared','after_prepared','before_cas','after_cas','after_confirmation','after_receipt'])
    args=p.parse_args()
    if args.logical and args.fixtures!=['tetris']:p.error('--logical requires --fixtures tetris')
    if not args.logical and 'subject' in args.fixtures and args.subject is None:p.error('--subject is required for the Subject fixture')
    args.out.mkdir(parents=True,exist_ok=False)
    rows=[]
    root=Path(__file__).resolve().parents[1]
    runner=root/('tests/beta2_logical_native_browser_storage_runner.mjs' if args.logical else 'tests/beta2_native_browser_kill_runner.mjs')
    phases=args.stages or ['before_prepared','after_prepared','before_cas','after_cas','after_confirmation','after_receipt']
    stopped=False
    for fixture in args.fixtures:
        if stopped:break
        for phase in phases:
            out=args.out/f'{fixture}-{phase}'
            command=['node',str(runner),str(args.stage.resolve()),str(out.resolve()),phase]
            fixture_path=(args.subject.resolve() if fixture=='subject' else
                root/'solutions/seamgrim_ui_mvp/samples'/('beta1_logical_tetris' if fixture=='tetris' else 'beta1_logical_counter')/'authored-project.json')
            command.extend([args.transaction] if args.logical else [str(fixture_path),args.transaction])
            # Do not parallelize: private Windows custody/Chrome CPU costs made
            # two prior concurrent observations expire before a rendezvous.
            # The child owns and cleans up only its own Native/Chrome handles.
            completed=subprocess.run(command,cwd=root,capture_output=True,text=True,encoding='utf8')
            report_path=out/'RESULT.json'
            report=json.loads(report_path.read_text(encoding='utf8')) if report_path.exists() else {}
            rows.append({'fixture':fixture,'stage':phase,'exit':completed.returncode,
                'verdict':report.get('verdict','MISSING_RESULT'),'report':str(report_path),
                'runner_sha256':report.get('runner_sha256'),
                'report_sha256':sha(report_path) if report_path.exists() else None,
                'native_sha256':report.get('native_sha256'),'wasm_sha256':report.get('wasm_sha256'),
                'actual_killed_pid':report.get('killed_pid'),'source_preserved':report.get('source_locator_bytes_preserved'),
                'actual_marker_matched':(out/'reached').is_file() and (out/'reached').read_text(encoding='utf8')==phase,
                'kill_exit':report.get('kill_exit'),
                'b2_04_prior_normal_match':report.get('b2_04_prior_normal_match'),
                'stdout':completed.stdout,'stderr':completed.stderr})
            def accepted(row):
                actual_kill=isinstance(row['actual_killed_pid'],int) and row['actual_killed_pid']>0 and isinstance(row['kill_exit'],dict) and row['kill_exit'].get('code')!=0
                return actual_kill and row['actual_marker_matched'] and row['source_preserved'] is True and row['exit']==0 and (row['verdict'].startswith('PASS_') or
                    (args.transaction=='next' and row['verdict']=='OBSERVED_NEXT_COLD_RECOVERY_REQUIRED_NO_PARTIAL_WORLD'))
            passed=sum(accepted(r) for r in rows)
            matrix={'claim':f'ACTUAL_CHROME_LOGICAL_{args.transaction.upper()}_TRANSACTION_TETRIS_KEYBOARD' if args.logical else f'ACTUAL_CHROME_{args.transaction.upper()}_TRANSACTION_SOURCE_PATH_NO_EXTERNAL_LOGICAL_INPUT',
                'policy':'R5.1 failclosed; next preCAS prior_normal_match=false remains raw observation, not fallback success',
                'runner_sha256':sha(runner),'pass':passed,'fail':len(rows)-passed,'rows':rows,
                'expected_rows':len(args.fixtures)*len(phases),'requested_phases':phases,
                'unrun_fixtures':[item for item in ['counter','tetris','subject'] if item not in args.fixtures],
                'not_covered':(['CLI three names','counter/Subject Native Workbench in this matrix'] if args.logical else ['Tetris keyboard logical Native Workbench'])+[
                    'subsequent transaction' if args.transaction=='first' else 'first transaction in this matrix',
                    'corrupt/missing/swapped custody','physical power loss','exact final Beta2 ZIP','Human']}
            (args.out/f'B2_BROWSER_{args.transaction.upper()}_MATRIX.partial.json').write_text(json.dumps(matrix,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
            print(json.dumps({'fixture':fixture,'stage':phase,'verdict':report.get('verdict'),'pass':passed,'fail':len(rows)-passed}),flush=True)
            if len(rows)>=2 and all(row['exit']!=0 for row in rows[-2:]):
                print('STOPPED_AFTER_TWO_FAILURES: inspect the exact evidence before repeating',flush=True)
                stopped=True;break
    if len(rows)!=len(args.fixtures)*len(phases) or any(not accepted(row) for row in rows):raise SystemExit(1)

if __name__=='__main__':main()
