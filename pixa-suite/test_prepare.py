import importlib.util,json,subprocess,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
class PreparationTests(unittest.TestCase):
 def test_catalog_is_unique_and_pinned(self):
  apps=p.CATALOG
  self.assertEqual(len(apps),14)
  existing=json.loads((p.ROOT/'catalog.json').read_text())['existing_products']
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
   self.assertEqual(data['product'],app['id']);self.assertEqual(len(data['chapters']),6);self.assertEqual(len(data['tour']),4)
   self.assertIn('not a complete reference',data['scope'])
 def test_runtime_changes_invalidate_source_receipt(self):
  spec=importlib.util.spec_from_file_location('build',p.ROOT/'build.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);source=root/'source';source.mkdir();runtime=root/'runtime';(runtime/'src').mkdir(parents=True)
   (runtime/'Cargo.toml').write_text('[package]\nname="fixture"\n')
   code=runtime/'src/lib.rs';code.write_text('pub fn version() -> u8 { 1 }')
   subprocess.run(['git','init',str(source)],capture_output=True,check=True)
   (source/'main.rs').write_text('fn main() {}');b.ROOT=root
   first=b.source_digest(source);code.write_text('pub fn version() -> u8 { 2 }')
   self.assertNotEqual(first,b.source_digest(source))
 def test_runtime_is_independent_inside_product_repository(self):
  import shutil
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);(root/'Cargo.toml').write_text('[workspace]\nmembers=[]\nresolver="3"\n')
   sdk=root/'pixa-suite/runtime';shutil.copytree(p.ROOT/'runtime',sdk,ignore=shutil.ignore_patterns('target'))
   cargo=shutil.which('cargo') or '/home/agent/.cargo/bin/cargo'
   result=subprocess.run([cargo,'+1.95.0','metadata','--no-deps','--format-version','1','--manifest-path',str(sdk/'Cargo.toml')],capture_output=True,text=True)
   self.assertEqual(result.returncode,0,result.stderr)
   self.assertEqual(Path(json.loads(result.stdout)['workspace_root']).resolve(),sdk.resolve())
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
