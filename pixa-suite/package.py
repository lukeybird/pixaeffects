#!/usr/bin/env python3
"""Package locally built Pixa development binaries, never publish or sign them."""
from pathlib import Path
import argparse, hashlib, json, plistlib, shutil, sys, tarfile, zipfile
from build import source_digest
ROOT=Path(__file__).resolve().parent

def package(app_id):
 apps=json.loads((ROOT/'catalog.json').read_text(encoding="utf-8"))['apps']
 app=next((a for a in apps if a['id']==app_id),None)
 if app is None:raise ValueError('Unknown product')
 if sys.platform not in {'darwin','win32','linux'}:raise ValueError('Unsupported platform')
 ext='.exe' if sys.platform=='win32' else ''
 binary=ROOT/'target/release'/(app['package']+ext)
 if not binary.is_file():raise FileNotFoundError('Build the release binary before packaging: '+str(binary))
 # Match the exact prepared source; a stale binary must not be labeled a new source build.
 receipt=ROOT/'build-logs'/(app_id+'-release.json')
 if not receipt.exists():raise RuntimeError('Missing successful release-build receipt; use build.py')
 data=json.loads(receipt.read_text(encoding="utf-8"))
 if data.get('access_mode') not in {'development','subscription'}:raise RuntimeError('Build receipt has no access mode; rebuild with current tooling.')
 if data.get('source_digest')!=source_digest(ROOT/'sources'/app['source']):raise RuntimeError('Prepared source changed after the build')
 if data.get('sha256')!=hashlib.sha256(binary.read_bytes()).hexdigest() or data.get('revision')!=app['revision']:raise RuntimeError('Binary does not match its build receipt')
 out=ROOT/'dist'/app_id/data['access_mode'];out.mkdir(parents=True,exist_ok=True)
 stage=out/'stage'
 if stage.exists():raise RuntimeError('Remove the previous staging directory before packaging')
 stage.mkdir()
 source=ROOT/'sources'/app['source']
 executable=stage/(app['name']+ext)
 if sys.platform=='darwin':
  contents=stage/(app['name']+'.app')/'Contents';(contents/'MacOS').mkdir(parents=True);(contents/'Resources').mkdir()
  executable=contents/'MacOS'/app['id']
  info={'CFBundleName':app['name'],'CFBundleDisplayName':app['name'],'CFBundleIdentifier':app['bundle_id'],'CFBundleExecutable':app['id'],'CFBundleShortVersionString':app['version'],'CFBundleVersion':app['version'],'CFBundlePackageType':'APPL','CFBundleIconFile':'app.icns','NSHighResolutionCapable':True}
  (contents/'Info.plist').write_bytes(plistlib.dumps(info))
  icons=list((source/'assets/app-icon').rglob('*.icns'))
  if not icons:raise RuntimeError('No prepared Mac app icon')
  shutil.copyfile(icons[0],contents/'Resources/app.icns')
 shutil.copy2(binary,executable)
 executable.chmod(0o755)
 notices=stage/'Licenses';notices.mkdir()
 for p in source.glob('*'):
  if p.is_file() and (p.name.startswith(('LICENSE','NOTICE')) or p.name=='ATTRIBUTION.md'):shutil.copy2(p,notices/p.name)
 (stage/'DEVELOPMENT_BUILD.txt').write_text('Unpublished development build. Not signed or notarized. Subscription enforcement and verified Pixa update integration are not yet complete. Do not distribute to customers.\n', encoding="utf-8")
 archive=out/(app_id+'-'+app['version']+'-'+sys.platform+'-development.zip')
 with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
  for p in stage.rglob('*'):
   if p.is_file():z.write(p,p.relative_to(stage))
 (Path(str(archive)+'.sha256')).write_text(hashlib.sha256(archive.read_bytes()).hexdigest()+'  '+archive.name+'\n', encoding="utf-8")
 print(archive)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('app');package(p.parse_args().app)
