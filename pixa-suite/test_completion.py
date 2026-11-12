"""Regression checks for candidate packaging and source-bound guide generation."""
import importlib.util,json,subprocess,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT))
import manuals
class CompletionTests(unittest.TestCase):
 def test_lock_resolution_precedes_fingerprint_and_build_is_locked(self):
  import os,shutil
  if sys.platform=='win32':self.skipTest('Synthetic Cargo fixture uses a Unix shebang')
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);shutil.copy2(ROOT/'build.py',root/'build.py')
   source=root/'sources/fixture';source.mkdir(parents=True)
   subprocess.run(['git','init',str(source)],capture_output=True,check=True)
   (source/'.pixa-prepared.json').write_text('{}')
   (source/'Cargo.lock').write_text('before resolution')
   (root/'catalog.json').write_text(json.dumps({'apps':[{'id':'pixafixture','source':'fixture','package':'fixture','revision':'a'*40,'rust_toolchain':'1.95.0','name':'Fixture'}]}))
   binary=root/'target/release/fixture';fake=root/'bin';fake.mkdir();cargo=fake/'cargo'
   cargo.write_text('#!'+sys.executable+'\nimport sys\nfrom pathlib import Path\nif "metadata" in sys.argv:\n Path("Cargo.lock").write_text("resolved before compilation")\n sys.exit(0)\nif "--locked" not in sys.argv or Path("Cargo.lock").read_text()!="resolved before compilation":sys.exit(42)\np=Path('+repr(str(binary))+');p.parent.mkdir(parents=True);p.write_bytes(b"binary")\n')
   cargo.chmod(0o755);env=os.environ.copy();env['PATH']=str(fake)+os.pathsep+env.get('PATH','')
   result=subprocess.run([sys.executable,str(root/'build.py'),'pixafixture','--release'],env=env,capture_output=True,text=True)
   self.assertEqual(result.returncode,0,result.stderr)
   receipt=json.loads((root/'build-logs/pixafixture-release.json').read_text())
   self.assertEqual(receipt['revision'],'a'*40)
   self.assertEqual((source/'Cargo.lock').read_text(),'resolved before compilation')

 def test_source_changed_during_build_cannot_get_success_receipt(self):
  import os,shutil
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);shutil.copy2(ROOT/'build.py',root/'build.py')
   source=root/'sources/fixture';source.mkdir(parents=True)
   subprocess.run(['git','init',str(source)],capture_output=True,check=True)
   (source/'.pixa-prepared.json').write_text('{}')
   code=source/'main.rs';code.write_text('before')
   (root/'catalog.json').write_text(json.dumps({'apps':[{'id':'pixafixture','source':'fixture','package':'fixture','revision':'a'*40,'rust_toolchain':'1.95.0','name':'Fixture'}]}))
   fake=root/'bin';fake.mkdir();cargo=fake/'cargo'
   binary=root/'target/release'/('fixture.exe' if sys.platform=='win32' else 'fixture')
   if sys.platform=='win32':self.skipTest('Synthetic executable fixture uses a Unix shebang')
   cargo.write_text('#!'+sys.executable+'\nimport sys\nfrom pathlib import Path\nif "build" not in sys.argv:sys.exit(0)\np=Path('+repr(str(binary))+');p.parent.mkdir(parents=True);p.write_bytes(b"binary")\nPath('+repr(str(code))+').write_text("changed during compilation")\n')
   cargo.chmod(0o755);env=os.environ.copy();env['PATH']=str(fake)+os.pathsep+env.get('PATH','')
   result=subprocess.run([sys.executable,str(root/'build.py'),'pixafixture','--release'],env=env,capture_output=True,text=True)
   self.assertNotEqual(result.returncode,0)
   self.assertIn('Source changed while building',result.stderr)
   self.assertFalse((root/'build-logs/pixafixture-release.json').exists())
 def test_control_guides_fail_when_control_disappears(self):
  app={'id':'pixalight','source':'lightcraft'}
  with tempfile.TemporaryDirectory() as td:
   source=Path(td);file=source/'crates/ui-egui/src/state/view.rs';file.parent.mkdir(parents=True);file.write_text('enum RightPanel { Edit }')
   with self.assertRaisesRegex(RuntimeError,'Manual control reference changed'):manuals.control_chapters(app,source)
 def test_anything_embeds_actual_beginner_and_expert_help(self):
  with tempfile.TemporaryDirectory() as td:
   source=Path(td);file=source/'crates/engine/src/tools/example.rs';file.parent.mkdir(parents=True)
   file.write_text('ToolSpec { id: "test_tool", title: "A tool", summary: "Exact source detail.", beginner: "Simple explanation.", toolset: Toolset::Core }')
   rows=manuals.anything_tools({'source':'craft-anything'},source)
   self.assertEqual(rows,[{'title':'A tool','body':'Simple explanation. Exact source detail.','tool_id':'test_tool'}])
 def test_candidate_packaging_rejects_development_and_wrong_host(self):
  spec=importlib.util.spec_from_file_location('candidate_package',ROOT/'package.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);m.ROOT=root
   (root/'catalog.json').write_text(json.dumps({'apps':[{'id':'pixafixture','source':'fixture','package':'fixture','revision':'a'*40}]}))
   binary=root/'target/release'/('fixture.exe' if sys.platform=='win32' else 'fixture');binary.parent.mkdir(parents=True);binary.write_bytes(b'fixture')
   receipt=root/'build-logs/pixafixture-release.json';receipt.parent.mkdir();receipt.write_text(json.dumps({'platform':sys.platform,'access_mode':'development'}))
   with self.assertRaisesRegex(RuntimeError,'account-gated'):m.package('pixafixture',True)
   receipt.write_text(json.dumps({'platform':'different-host','access_mode':'subscription'}))
   with self.assertRaisesRegex(RuntimeError,'platform'):m.package('pixafixture',True)

 def test_candidate_keeps_asset_license_texts_and_attributions(self):
  import hashlib,tarfile,zipfile
  from unittest.mock import patch
  spec=importlib.util.spec_from_file_location('license_package',ROOT/'package.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
  with tempfile.TemporaryDirectory() as temporary:
   root=Path(temporary);module.ROOT=root
   app={'id':'pixavector','name':'PixaVector','source':'vectorcraft','package':'vectorcraft','version':'0.1.1','revision':'a'*40}
   (root/'catalog.json').write_text(json.dumps({'apps':[app]}))
   source=root/'sources/vectorcraft';(source/'assets/fonts').mkdir(parents=True)
   (source/'assets/app-icon').mkdir(parents=True)
   (source/'LICENSE-MIT').write_text('Synthetic retained license')
   (source/'ASSETS.md').write_text('Synthetic asset index')
   (source/'assets/fonts/OFL-Fixture.txt').write_text('Synthetic font license')
   (source/'assets/app-icon/fixture.png.attribution').write_text('Original synthetic icon')
   binary=root/'target/release'/('vectorcraft.exe' if sys.platform=='win32' else 'vectorcraft');binary.parent.mkdir(parents=True);binary.write_bytes(b'fixture')
   receipt=root/'build-logs/pixavector-release.json';receipt.parent.mkdir()
   receipt.write_text(json.dumps({'platform':sys.platform,'access_mode':'subscription','revision':app['revision'],'source_digest':'fixture','sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}))
   if sys.platform=='darwin':self.skipTest('Synthetic fixture has no real Mac icon')
   with patch.object(module,'source_digest',return_value='fixture'):
    module.package('pixavector',True)
   archive=next((root/'dist/pixavector/candidate').glob('*.tar.gz' if sys.platform=='linux' else '*.zip'))
   if sys.platform=='linux':
    with tarfile.open(archive) as bundle:names=bundle.getnames()
   else:
    with zipfile.ZipFile(archive) as bundle:names=bundle.namelist()
   for ending in ['Licenses/ASSETS.md','Licenses/assets/fonts/OFL-Fixture.txt','Licenses/assets/app-icon/fixture.png.attribution']:
    self.assertTrue(any(name.endswith(ending) for name in names),ending)

if __name__=='__main__':unittest.main()
