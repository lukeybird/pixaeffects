import hashlib,importlib.util,json,plistlib,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('finish',Path(__file__).resolve().parents[1]/'finish_macos.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Acceptance(unittest.TestCase):
 def fixture(self,t):
  app=Path(t)/'Pixa Launcher.app';(app/'Contents/MacOS').mkdir(parents=True);(app/'Contents/MacOS/pixa-launcher').write_bytes(b'fixture')
  (app/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'com.aiwebdesignfirm.pixalauncher','CFBundleShortVersionString':'0.2.0','CFBundleExecutable':'pixa-launcher'}))
  digest=hashlib.sha256(b'fixture').hexdigest();report={'executable_sha256':digest,'architecture':'universal','source_dirty':False,'source_commit':'a'*40};record={'executable_sha256':digest,'source_commit':'a'*40,'checks':{k:'passed' for k in m.CHECKS}}
  rp=Path(t)/'receipt.json';ap=Path(t)/'acceptance.json';rp.write_text(json.dumps(report));ap.write_text(json.dumps(record));return app,rp,ap,record
 def test_exact_owner_acceptance_is_bound_to_binary(self):
  with tempfile.TemporaryDirectory() as t:
   app,rp,ap,_=self.fixture(t);self.assertEqual(m.acceptance(app,rp,ap)['source_commit'],'a'*40)
   (app/'Contents/MacOS/pixa-launcher').write_bytes(b'changed')
   with self.assertRaises(ValueError):m.acceptance(app,rp,ap)
 def test_unfinished_checks_and_foreign_source_are_refused(self):
  with tempfile.TemporaryDirectory() as t:
   app,rp,ap,record=self.fixture(t);record['checks']['install_and_open']='not-run';ap.write_text(json.dumps(record))
   with self.assertRaises(ValueError):m.acceptance(app,rp,ap)
   record['checks']['install_and_open']='passed';record['source_commit']='b'*40;ap.write_text(json.dumps(record))
   with self.assertRaises(ValueError):m.acceptance(app,rp,ap)
if __name__=='__main__':unittest.main()
