import importlib.util,json,plistlib,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('package',Path(__file__).resolve().parents[1]/'package_macos.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
class Packaging(unittest.TestCase):
 def test_existing_output_is_refused_without_changes(self):
  with tempfile.TemporaryDirectory() as t:
   p=Path(t);(p/'keep').write_text('original')
   with self.assertRaises(RuntimeError):module.package(p,[])
   self.assertEqual((p/'keep').read_text(),'original')
 def test_candidate_metadata_is_explicitly_unsigned(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'source';root.mkdir()
   for name in ['README.md','ATTRIBUTION.md','THIRD_PARTY_LICENSES.txt','catalog.json']:(root/name).write_text('fixture')
   calls=[]
   def run(*args):
    args=list(map(str,args));calls.append(args)
    if args[0].endswith('lipo'):Path(args[-1]).write_bytes(b'universal-fixture')
    if args[0].endswith('iconutil'):Path(args[-1]).write_bytes(b'icon-fixture')
   with patch.object(module,'ROOT',root),patch.object(module,'run',run),patch.object(module.subprocess,'check_output',side_effect=['source-sha\n','']):
    app=module.package(Path(t)/'output',['arm','intel'])
   info=plistlib.loads((app/'Contents/Info.plist').read_bytes());self.assertEqual(info['CFBundleIdentifier'],'com.aiwebdesignfirm.pixalauncher')
   receipt=json.loads((app.parent/'CANDIDATE.json').read_text());self.assertFalse(receipt['notarized']);self.assertFalse(receipt['apple_signed']);self.assertEqual(receipt['source_commit'],'source-sha')
   self.assertTrue(any(c[:3]==['/usr/bin/lipo','-create','arm'] for c in calls));self.assertFalse(any('notarytool' in c for c in calls))
if __name__=='__main__':unittest.main()
