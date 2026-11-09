import importlib.util,json,subprocess,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
class PreparationTests(unittest.TestCase):
 def test_catalog_is_unique_and_pinned(self):
  apps=p.CATALOG
  self.assertEqual(len(apps),14)
  for key in ['id','name','bundle_id','source']:self.assertEqual(len({a[key] for a in apps}),14)
  for a in apps:
   self.assertRegex(a['revision'],r'^[0-9a-f]{40}$')
   self.assertTrue(a['bundle_id'].startswith('com.aiwebdesignfirm.pixa'))
   self.assertEqual(a['distribution'],'unpublished')
 def test_rust_balancer_ignores_strings_and_comments(self):
  s='(Thing { text: "(()", raw: r#"())"#, /* ) */ call: foo() }) trailing'
  self.assertEqual(s[p.matching_paren(s,0)+1:],' trailing')
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
