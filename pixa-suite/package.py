#!/usr/bin/env python3
"""Package receipt-bound development or unsigned account-gated candidates; never publish or sign."""
from pathlib import Path
import argparse, hashlib, json, platform, plistlib, shutil, sys, tarfile, zipfile
from build import source_digest
ROOT=Path(__file__).resolve().parent

def package(app_id, candidate=False):
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
 if data.get('platform') != sys.platform:raise RuntimeError('Build platform does not match packaging host')
 if candidate and data.get('access_mode')!='subscription':raise RuntimeError('Candidate packaging requires an account-gated release build; use build.py --release --access-mode subscription')
 if data.get('access_mode') not in {'development','subscription'}:raise RuntimeError('Build receipt has no access mode; rebuild with current tooling.')
 if data.get('source_digest')!=source_digest(ROOT/'sources'/app['source']):raise RuntimeError('Prepared source changed after the build')
 if data.get('sha256')!=hashlib.sha256(binary.read_bytes()).hexdigest() or data.get('revision')!=app['revision']:raise RuntimeError('Binary does not match its build receipt')
 channel='candidate' if candidate else 'development'
 out=ROOT/'dist'/app_id/channel;out.mkdir(parents=True,exist_ok=True)
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
 from manuals import markdown
 (stage/(app['name']+'-Manual.md')).write_text(markdown(app,source),encoding='utf-8')
 notice='Unsigned account-gated candidate. Apple signing, platform acceptance and release approval remain separate. Not a public installer. Do not distribute to customers.\n' if candidate else 'Unpublished development build. Editing is available without production account enforcement. Not signed or notarized. Do not distribute to customers.\n'
 (stage/('CANDIDATE_BUILD.txt' if candidate else 'DEVELOPMENT_BUILD.txt')).write_text(notice,encoding='utf-8')
 architecture=platform.machine().lower()
 if architecture in {'amd64','x86_64'}:architecture='x86_64'
 elif architecture in {'arm64','aarch64'}:architecture='arm64'
 else:raise RuntimeError('Unsupported host architecture: '+architecture)
 # Host-only packaging never claims a universal Mac bundle.
 label={'darwin':'macos','win32':'windows','linux':'linux'}[sys.platform]+'-'+architecture
 extension='.tar.gz' if sys.platform=='linux' else '.zip'
 archive=out/(app_id+'-'+app['version']+'-'+label+'-'+channel+extension)
 if sys.platform=='linux':
  with tarfile.open(archive,'w:gz') as z:z.add(stage,arcname=app['name'])
 else:
  with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
   for p in sorted(stage.rglob('*')):
    if p.is_file():z.write(p,p.relative_to(stage))
 digest=hashlib.sha256(archive.read_bytes()).hexdigest()
 (Path(str(archive)+'.sha256')).write_text(digest+'  '+archive.name+'\n', encoding='utf-8')
 (Path(str(archive)+'.json')).write_text(json.dumps({**data,'archive':archive.name,'archive_sha256':digest,'architecture':architecture,'channel':channel,'signed':False,'release_ready':False},indent=2)+'\n',encoding='utf-8')
 print(archive)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('app');p.add_argument('--candidate',action='store_true');a=p.parse_args();package(a.app,a.candidate)
