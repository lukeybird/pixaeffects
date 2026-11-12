import importlib.util,json,subprocess,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
class PreparationTests(unittest.TestCase):
 def test_catalog_is_unique_and_pinned(self):
  apps=p.CATALOG
  self.assertEqual(len(apps),14)
  existing=json.loads((p.ROOT/'catalog.json').read_text(encoding="utf-8"))['existing_products']
  self.assertEqual(len({a['id'] for a in apps+existing}),16)
  for key in ['id','name','bundle_id','source']:self.assertEqual(len({a[key] for a in apps}),14)
  for a in apps:
   self.assertRegex(a['revision'],r'^[0-9a-f]{40}$')
   self.assertTrue(a['bundle_id'].startswith('com.aiwebdesignfirm.pixa'))
   self.assertEqual(a['distribution'],'unpublished')
 def test_rust_balancer_ignores_strings_and_comments(self):
  s='(Thing { text: "(()", raw: r#"())"#, /* ) */ call: foo() }) trailing'
  self.assertEqual(s[p.matching_paren(s,0)+1:],' trailing')
 def test_module_insertion_preserves_mac_only_condition(self):
  original='//! App docs\n#![deny(unsafe_code)]\n#[cfg(target_os = "macos")]\nmod apple_events;\nuse eframe::App;\n'
  result=p.insert_shell_module(original)
  self.assertIn('mod pixa_shell;\n#[cfg(target_os = "macos")]\nmod apple_events;',result)
  self.assertTrue(result.startswith('//! App docs\n#![deny(unsafe_code)]'))
 def test_all_products_have_product_specific_starter_guides(self):
  spec=importlib.util.spec_from_file_location('manuals',p.ROOT/'manuals.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
  for app in p.CATALOG:
   data=m.guide(app)
   self.assertEqual(data['product'],app['id']);self.assertEqual(len(data['chapters']),12);self.assertEqual(len(data['tour']),4)
   self.assertIn('not a complete reference',data['scope'])
 def test_runtime_changes_invalidate_source_receipt(self):
  spec=importlib.util.spec_from_file_location('build',p.ROOT/'build.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);source=root/'source';source.mkdir();runtime=root/'runtime';(runtime/'src').mkdir(parents=True)
   (runtime/'Cargo.toml').write_text('[package]\nname="fixture"\n', encoding="utf-8")
   code=runtime/'src/lib.rs';code.write_text('pub fn version() -> u8 { 1 }', encoding="utf-8")
   subprocess.run(['git','init',str(source)],capture_output=True,check=True)
   (source/'main.rs').write_text('fn main() {}', encoding="utf-8");b.ROOT=root
   first=b.source_digest(source);code.write_text('pub fn version() -> u8 { 2 }', encoding="utf-8")
   self.assertNotEqual(first,b.source_digest(source))
 def test_runtime_is_independent_inside_product_repository(self):
  import shutil
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);(root/'Cargo.toml').write_text('[workspace]\nmembers=[]\nresolver="3"\n', encoding="utf-8")
   sdk=root/'pixa-suite/runtime';shutil.copytree(p.ROOT/'runtime',sdk,ignore=shutil.ignore_patterns('target'))
   cargo=shutil.which('cargo') or '/home/agent/.cargo/bin/cargo'
   result=subprocess.run([cargo,'+1.95.0','metadata','--no-deps','--format-version','1','--manifest-path',str(sdk/'Cargo.toml')],capture_output=True,text=True, encoding="utf-8")
   self.assertEqual(result.returncode,0,result.stderr)
   self.assertEqual(Path(json.loads(result.stdout)['workspace_root']).resolve(),sdk.resolve())
 def test_preparation_preserves_unicode_with_utf8_mode_disabled(self):
  import os
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);app=dict(p.CATALOG[0]);app['id']='pixafixture';checkout=root/app['source'];(checkout/'apps'/app['source']/'src').mkdir(parents=True)
   (checkout/'Cargo.toml').write_text('[workspace]\nmembers=["apps/vectorcraft"]\n[workspace.package]\nversion = "1.0.0"\n# Unicode source: Ω —\n',encoding='utf-8')
   (checkout/'apps/vectorcraft/Cargo.toml').write_text('[package]\nname="vectorcraft"\nversion.workspace=true\n[dependencies]\n',encoding='utf-8')
   (checkout/'apps/vectorcraft/src/main.rs').write_text('// Preserve Ω — in source\nfn main() { let _ = Ok(Box::new(App {})); }\n',encoding='utf-8')
   subprocess.run(['git','init',str(checkout)],capture_output=True,check=True)
   subprocess.run(['git','add','.'],cwd=checkout,capture_output=True,check=True)
   subprocess.run(['git','-c','user.name=Test','-c','user.email=test@example.invalid','commit','-m','fixture'],cwd=checkout,capture_output=True,check=True)
   app['revision']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=checkout,text=True,encoding='utf-8').strip()
   environment=os.environ.copy();environment.update({'LC_ALL':'C','PYTHONCOERCECLOCALE':'0','PYTHONUTF8':'0'})
   import sys
   code="import sys,json;from pathlib import Path;sys.path.insert(0,sys.argv[1]);import prepare,manuals;app=json.loads(sys.argv[2]);manuals.WORKFLOWS[app['id']]=manuals.WORKFLOWS['pixavector'];prepare.prepare(app,Path(sys.argv[3]))"
   result=subprocess.run([sys.executable,'-X','utf8=0','-c',code,str(p.ROOT),json.dumps(app),str(root)],env=environment,capture_output=True,text=True,encoding='utf-8')
   self.assertEqual(result.returncode,0,result.stderr)
   self.assertIn('Ω —',(checkout/'apps/vectorcraft/src/main.rs').read_text(encoding='utf-8'))
 def test_runtime_dependency_path_handles_windows_cross_drive_temp(self):
  import sys,ntpath
  from unittest.mock import patch
  sys.path.insert(0,str(p.ROOT))
  import integrations
  with self.assertRaises(ValueError):ntpath.relpath('D:/repo/runtime','C:/Temp/project')
  with patch('integrations.os.path.relpath',side_effect=ValueError('different drive')):
   result=integrations.dependency_path(p.ROOT/'runtime',Path('/temporary/project'))
  self.assertEqual(result,(p.ROOT/'runtime').resolve().as_posix())
 def test_recovery_pilots_have_native_examples_and_control_gates(self):
  import sys
  sys.path.insert(0,str(p.ROOT))
  from integrations import PILOTS
  examples={'pixafilm':'film','pixatexture':'texture','pixadesign':'design','pixadeck':'deck','pixavector':'vector','pixagrid':'grid','pixaword':'word','pixapdf':'pdf','pixamaterial':'material','pixacad':'cad','pixalight':'light','pixasound':'sound','pixainterface':'interface','pixaanything':'anything'}
  self.assertEqual(PILOTS,set(examples))
  for value in examples.values():self.assertTrue((p.ROOT/'recovery-checks'/(value+'.rs')).is_file())
 def test_display_branding_preserves_internal_identifiers(self):
  import sys
  sys.path.insert(0,str(p.ROOT));from branding import display_strings
  original='fn editInPhotoCraft() { run("photo.editInPhotoCraft"); label("Edit in PhotoCraft"); label("PixaDesign on GitHub…"); }'
  updated=display_strings(original,{'PhotoCraft':'PixaShop'})
  self.assertIn('fn editInPhotoCraft()',updated)
  self.assertIn('"photo.editInPhotoCraft"',updated)
  self.assertIn('"Edit in PixaShop"',updated)
  self.assertIn('"PixaDesign help…"',updated)
 def test_design_tool_catalog_drift_fails_instead_of_silent_incomplete_tour(self):
  import sys
  sys.path.insert(0,str(p.ROOT));from manuals import tool_chapters
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);file=root/'crates/tools/src/catalog.rs';file.parent.mkdir(parents=True)
   file.write_text('t("unknownTool", "Unknown tool", None, "icon"),',encoding='utf-8')
   with self.assertRaisesRegex(RuntimeError,'Missing tool explanation'):tool_chapters({'id':'pixadesign'},root)
 def test_unknown_product_subscription_build_never_reaches_cargo(self):
  import sys
  result=subprocess.run([sys.executable,str(p.ROOT/'build.py'),'pixa-does-not-exist','--access-mode','subscription'],capture_output=True,text=True,encoding='utf-8')
  self.assertEqual(result.returncode,2)
  self.assertIn('Unknown app',result.stderr)
 def test_unbalanced_expression_fails(self):
  with self.assertRaises(ValueError):p.matching_paren('(foo()',0)
 def test_icon_generation_is_deterministic_and_distinct(self):
  hashes=[]
  for a in p.CATALOG:
   first=p.icon(a,128).tobytes()
   self.assertEqual(first,p.icon(a,128).tobytes())
   hashes.append(first)
  self.assertEqual(len(set(hashes)),14)
 def test_wrong_pin_never_modifies_checkout(self):
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);a=p.CATALOG[0];checkout=root/a['source'];checkout.mkdir()
   subprocess.run(['git','init',str(checkout)],capture_output=True,check=True)
   subprocess.run(['git','-c','user.name=Test','-c','user.email=test@example.invalid','commit','--allow-empty','-m','fixture'],cwd=checkout,capture_output=True,check=True)
   before=subprocess.check_output(['git','status','--porcelain'],cwd=checkout)
   with self.assertRaisesRegex(RuntimeError,'unexpected source revision'):p.prepare(a,root)
   self.assertEqual(before,subprocess.check_output(['git','status','--porcelain'],cwd=checkout))
if __name__=='__main__':unittest.main()
