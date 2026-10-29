"""No fabricated runtime output: check packaging refusal and verdict honesty."""
from pathlib import Path
import hashlib,importlib.util,json,sys,tempfile,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'tools'))
import v2_candidate_archive as archive
import v3_notebook_verdict as verdict

class CandidateGuards(unittest.TestCase):
    def test_v3_requires_all_seven_rows_and_keeps_missing_distinct(self):
        rows=[{'id':'M'+str(n),'supported':'NOT_RUN_PENDING_SUPPLIER','limits':'not verified'} for n in range(1,8)]
        self.assertEqual(archive.support_rows({'support':rows},'V3'),rows)
        with self.assertRaises(archive.PackageError):archive.support_rows({'support':rows[:-1]},'V3')
        with self.assertRaises(archive.PackageError):archive.support_rows({'support':rows},'V2')
    def test_v2_original_rows_remain_accepted(self):
        rows=[{'id':f'V2-{n:02}','supported':'existing','limits':'retained'} for n in range(1,7)]
        self.assertEqual(archive.support_rows({'support':rows},'V2'),rows)
    def test_build_source_does_not_adopt_an_unrelated_history_field(self):
        record={'compiled_runtime_source_commit':'a'*40,'source':'b'*40}
        self.assertEqual(archive.build_source(record,'/compiled_runtime_source_commit'),'a'*40)
        with self.assertRaises(archive.PackageError):archive.build_source(record,'/source')
    def test_adapter_pass_flag_cannot_replace_missing_observations(self):
        with tempfile.TemporaryDirectory() as name:
            base=Path(name);script=base/'wrong.py';script.write_text('print(\'{"status":"PASS"}\')\n',encoding='utf-8')
            entry={'script':script.name,'sha256':hashlib.sha256(script.read_bytes()).hexdigest(),'executable':sys.executable}
            self.assertEqual(verdict.run_adapter(entry,base,base,'M1')['status'],'FAIL')
            entry['sha256']='0'*64
            with self.assertRaises(ValueError):verdict.run_adapter(entry,base,base,'M1')

if __name__=='__main__':unittest.main()
