import plistlib,tempfile,unittest
from pathlib import Path
from macos_candidate import validate_bundle

class BundleValidation(unittest.TestCase):
 def bundle(self,root,**changes):
  app=root/'PixaVector.app';contents=app/'Contents';(contents/'MacOS').mkdir(parents=True)
  data={'CFBundleIdentifier':'com.aiwebdesignfirm.pixavector','CFBundleShortVersionString':'0.1.0','CFBundleExecutable':'pixavector'};data.update(changes)
  (contents/'Info.plist').write_bytes(plistlib.dumps(data));(contents/'MacOS/pixavector').write_bytes(b'candidate')
  return app
 def product(self):return {'id':'pixavector','bundle_id':'com.aiwebdesignfirm.pixavector','version':'0.1.0'}
 def test_correct_identity(self):
  with tempfile.TemporaryDirectory() as td:
   app=self.bundle(Path(td));self.assertEqual(validate_bundle(app,self.product()),app/'Contents/MacOS/pixavector')
 def test_wrong_product_rejected(self):
  with tempfile.TemporaryDirectory() as td:
   app=self.bundle(Path(td),CFBundleIdentifier='com.other.editor')
   with self.assertRaisesRegex(ValueError,'identity'):validate_bundle(app,self.product())
 def test_path_traversal_rejected(self):
  with tempfile.TemporaryDirectory() as td:
   app=self.bundle(Path(td),CFBundleExecutable='../../unrelated')
   with self.assertRaisesRegex(ValueError,'executable'):validate_bundle(app,self.product())
 def test_external_symlink_rejected(self):
  with tempfile.TemporaryDirectory() as td:
   app=self.bundle(Path(td))
   try:(app/'Contents/outside').symlink_to(Path(td))
   except OSError:self.skipTest('Creating symlinks requires an OS permission unavailable to this test user')
   with self.assertRaisesRegex(ValueError,'symlink'):validate_bundle(app,self.product())
if __name__=='__main__':unittest.main()
