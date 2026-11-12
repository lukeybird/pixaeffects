"""Film microphone preflight and minimal signing arguments, without invoking Mac tools."""
import importlib.util, json, hashlib, plistlib, tempfile, unittest, zipfile
from unittest.mock import patch
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('film_signing',ROOT/'release-tools/macos_candidate.py')
signing=importlib.util.module_from_spec(spec);spec.loader.exec_module(signing)

class FilmSigningTests(unittest.TestCase):
 def test_film_bundle_requires_a_real_microphone_purpose(self):
  product={'id':'pixafilm','version':'0.1.1','bundle_id':'com.aiwebdesignfirm.pixafilm'}
  with tempfile.TemporaryDirectory() as td:
   app=Path(td)/'PixaFilm.app';contents=app/'Contents';(contents/'MacOS').mkdir(parents=True)
   binary=contents/'MacOS/pixafilm';binary.write_bytes(b'synthetic, not Mach-O')
   info={'CFBundleIdentifier':product['bundle_id'],'CFBundleShortVersionString':'0.1.1','CFBundleExecutable':'pixafilm'}
   plist=contents/'Info.plist';plist.write_bytes(plistlib.dumps(info))
   with self.assertRaisesRegex(ValueError,'NSMicrophoneUsageDescription'):signing.validate_bundle(app,product)
   info['NSMicrophoneUsageDescription']='Record voice-over onto the timeline.';plist.write_bytes(plistlib.dumps(info))
   self.assertEqual(signing.validate_bundle(app,product),binary)

 def test_only_film_main_bundle_gets_audio_input_entitlement(self):
  with tempfile.TemporaryDirectory() as td:
   temporary=Path(td);app=temporary/'PixaFilm.app';nested=app/'Contents/Frameworks/fixture.dylib'
   command=signing.signing_command('Developer ID Application: Fixture',app,{'id':'pixafilm'},app,temporary)
   self.assertEqual(command[-1],app)
   plist=command[command.index('--entitlements')+1]
   self.assertEqual(plistlib.loads(plist.read_bytes()),{'com.apple.security.device.audio-input':True})
   for product,target in [({'id':'pixavector'},app),({'id':'pixafilm'},nested)]:
    self.assertNotIn('--entitlements',signing.signing_command('Fixture',target,product,app,temporary))

 def test_mac_candidate_package_contains_the_microphone_purpose(self):
  spec=importlib.util.spec_from_file_location('film_package',ROOT/'pixa-suite/package.py')
  packaging=importlib.util.module_from_spec(spec);spec.loader.exec_module(packaging)
  with tempfile.TemporaryDirectory() as td:
   root=Path(td);packaging.ROOT=root
   product={'id':'pixafilm','name':'PixaFilm','source':'filmcraft','package':'filmcraft','version':'0.1.1','bundle_id':'com.aiwebdesignfirm.pixafilm','revision':'a'*40}
   (root/'catalog.json').write_text(json.dumps({'apps':[product]}))
   source=root/'sources/filmcraft';icons=source/'assets/app-icon';icons.mkdir(parents=True)
   (icons/'fixture.icns').write_bytes(b'synthetic icon');(source/'LICENSE-MIT').write_text('Synthetic retained license')
   binary=root/'target/release/filmcraft';binary.parent.mkdir(parents=True);binary.write_bytes(b'synthetic app')
   receipt=root/'build-logs/pixafilm-release.json';receipt.parent.mkdir()
   receipt.write_text(json.dumps({'platform':'darwin','access_mode':'subscription','source_digest':'fixture','revision':product['revision'],'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}))
   with patch.object(packaging.sys,'platform','darwin'),patch.object(packaging,'source_digest',return_value='fixture'):
    packaging.package('pixafilm',True)
   archive=next((root/'dist/pixafilm/candidate').glob('*.zip'))
   with zipfile.ZipFile(archive) as bundle:
    info=plistlib.loads(bundle.read('PixaFilm.app/Contents/Info.plist'))
   self.assertEqual(info['NSMicrophoneUsageDescription'],'PixaFilm uses the microphone to record voice-over onto the timeline.')
   self.assertEqual(info['CFBundleIdentifier'],product['bundle_id'])

if __name__=='__main__':unittest.main()
