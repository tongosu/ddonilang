"""Actual offline notice pipeline, immutable baseline, four nonredundant red inputs."""
import argparse,copy,hashlib,importlib.util,json,shutil,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import generate_public_candidate_sbom as sbom
from alpha2_completion_archive import read_notice_bundle
sha=lambda data:hashlib.sha256(data).hexdigest()

def main():
 p=argparse.ArgumentParser();p.add_argument('--supplement',type=Path,required=True);p.add_argument('--expected-sha256',required=True)
 p.add_argument('--baseline-notices',type=Path,required=True);p.add_argument('--notices',type=Path,required=True);p.add_argument('--out',type=Path,required=True);args=p.parse_args()
 assert not args.out.exists();args.out.mkdir(parents=True)
 metadata=sbom.cargo_metadata(ROOT);inventory=sbom.build_inventory(ROOT,metadata)
 original=args.supplement.read_bytes();assert sha(original)==args.expected_sha256
 source=json.loads(original)
 baseline_raw=(args.baseline_notices/'THIRD_PARTY_NOTICES.json').read_bytes()
 baseline,_=read_notice_bundle(args.baseline_notices,inventory['source_lock_sha256'],sha(baseline_raw))
 default=sbom.build_license_bundle(metadata,inventory)
 assert set(default)==set(baseline),'default notice membership changed'
 assert all(default[name]==baseline[name] for name in default),'default notice bytes changed'
 assert json.loads(default['THIRD_PARTY_NOTICES.json'])['missing_text_count']==4
 files=sbom.build_license_bundle(metadata,inventory,args.supplement,args.expected_sha256)
 assert files==sbom.build_license_bundle(metadata,inventory,args.supplement,args.expected_sha256),'supplement repeat differs'
 admitted,_=read_notice_bundle(args.notices,inventory['source_lock_sha256'],sha((args.notices/'THIRD_PARTY_NOTICES.json').read_bytes()))
 assert files==admitted,'actual generated notice bundle differs from replay'
 notices=json.loads(files['THIRD_PARTY_NOTICES.json'])
 assert notices['missing_text_count']==0 and notices['license_review']=='NOT_RUN'
 assert notices['upstream_supplemented_package_count']==4
 rows=[]
 def rejected(label,path,expected,code):
  try:sbom.build_license_bundle(metadata,inventory,path,expected)
  except ValueError as error:
   assert str(error)==code,(label,str(error),code)
   rows.append({'case':label,'verdict':'EXPECTED_REJECTION','code':code})
  else:raise AssertionError(label+' admitted invalid text source')
 rejected('wrong_external_source_digest',args.supplement,'0'*64,'E_LICENSE_SUPPLEMENT_SOURCE_HASH')
 for label in ['foreign_lock','path_escape','tampered_text_recomputed_sha256']:
  case=args.out/label;shutil.copytree(args.supplement.parent,case)
  changed=copy.deepcopy(source)
  if label=='foreign_lock':
   changed['source_lock_sha256']='0'*64;code='E_LICENSE_SUPPLEMENT_BINDING'
  elif label=='path_escape':
   changed['packages'][0]['files'][0]['file']='../outside-notice';code='E_LICENSE_SUPPLEMENT_PATH'
  else:
   entry=changed['packages'][0]['files'][0];file=case/entry['file'];data=file.read_bytes()+b'\n';file.write_bytes(data)
   entry.update(bytes=len(data),sha256=sha(data));code='E_LICENSE_SUPPLEMENT_HASH'
  encoded=(json.dumps(changed,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode()
  manifest=case/'manifest.json';manifest.write_bytes(encoded)
  rejected(label,manifest,sha(encoded),code)
 report={'claim':'OFFLINE_LICENSE_TEXT_PROVENANCE_AND_BUILD_NOT_LEGAL_OR_RELEASE_PASS','verdict':'PASS',
   'source_lock_sha256':inventory['source_lock_sha256'],'source_manifest_sha256':args.expected_sha256,
   'default_bundle_byte_identical':True,'default_missing_text_count':4,'supplement_missing_text_count':0,
   'supplemented_package_count':4,'actual_bundled_files_verified':len(admitted),
   'notice_manifest_sha256':sha(files['THIRD_PARTY_NOTICES.json']),'four_adversarial_inputs':rows,
   'legal_review':'NOT_RUN','license_selection':'NOT_MADE'}
 (args.out/'RESULT.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
