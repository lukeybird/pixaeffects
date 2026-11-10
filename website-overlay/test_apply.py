import hashlib,importlib.util,json,tempfile,unittest
from pathlib import Path
import apply_website_overlay as patch

class WebsitePatch(unittest.TestCase):
 def fixture(self,root):
  for name in patch.REQUIRED:
   p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('preserved source')
  (root/'package.json').write_text(json.dumps({'dependencies':{'next':'15'}}))
 def test_incomplete_checkout_is_refused(self):
  with tempfile.TemporaryDirectory() as td:
   with self.assertRaisesRegex(ValueError,'Incomplete'):patch.plan(Path(td))
 def test_billing_and_builder_preserved_with_backup(self):
  with tempfile.TemporaryDirectory() as td:
   target=Path(td);self.fixture(target)
   before={p:(target/p).read_bytes() for p in patch.REQUIRED if '/api/' in p or '/builder/' in p}
   (target/'app/page.tsx').write_text('old homepage')
   backup=patch.apply(target)
   try:
    self.assertEqual((backup/'app/page.tsx').read_text(),'old homepage')
    self.assertTrue((target/'app/products/[slug]/page.tsx').is_file())
    for p,value in before.items():self.assertEqual((target/p).read_bytes(),value)
   finally:
    import shutil;shutil.rmtree(backup)
if __name__=='__main__':unittest.main()
