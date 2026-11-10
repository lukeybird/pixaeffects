#!/usr/bin/env python3
"""Sign and notarize a local Mac test candidate. Never upload a public release or keys."""
from pathlib import Path
import argparse, hashlib, json, plistlib, shutil, subprocess, sys, tempfile
ROOT=Path(__file__).resolve().parent.parent/'pixa-suite'

def validate_bundle(app,product):
 if app.suffix!='.app' or not app.is_dir():raise ValueError('Select an existing .app bundle')
 with (app/'Contents/Info.plist').open('rb') as f:info=plistlib.load(f)
 for field,expected in [('CFBundleIdentifier',product['bundle_id']),('CFBundleShortVersionString',product['version'])]:
  if info.get(field)!=expected:raise ValueError('Bundle identity/version does not match the product catalog: '+field)
 executable=info.get('CFBundleExecutable')
 if not isinstance(executable,str) or Path(executable).name!=executable or executable in {'.','..',''}:raise ValueError('Invalid bundle executable')
 binary=app/'Contents/MacOS'/executable
 if not binary.is_file():raise ValueError('Bundle executable is missing')
 for file in app.rglob('*'):
  if file.is_symlink() and not file.resolve().is_relative_to(app.resolve()):raise ValueError('Bundle contains a symlink outside itself')
 return binary

def run(args,capture=False):
 return subprocess.run([str(a) for a in args],check=True,text=True,capture_output=capture)

def macho(file):
 if not file.is_file() or file.is_symlink():return False
 with file.open('rb') as f:magic=f.read(4)
 return magic in {bytes.fromhex(x) for x in ('feedface','cefaedfe','feedfacf','cffaedfe','cafebabe','bebafeca','cafebabf','bfbafeca')}

def main():
 p=argparse.ArgumentParser(description=__doc__)
 p.add_argument('product');p.add_argument('app',type=Path)
 p.add_argument('--identity',required=True,help='Developer ID Application identity from the local Keychain')
 p.add_argument('--profile',default='PixaShop-Notary',help='Existing local notarytool Keychain profile; never a password')
 p.add_argument('--architecture',choices=['universal','arm64','x86_64'],default='universal')
 p.add_argument('--output',type=Path,required=True)
 p.add_argument('--notices',type=Path,help='Licenses folder from the matching package.py stage')
 args=p.parse_args()
 if sys.platform!='darwin':p.error('Signing must run on the Mac holding the private signing identity')
 products=json.loads((ROOT/'catalog.json').read_text())['apps']
 product=next((a for a in products if a['id']==args.product),None)
 if not product:p.error('Unknown suite product; use the existing Shop/Effects release scripts for those products')
 if not args.identity.startswith('Developer ID Application: '):p.error('Use a Developer ID Application signing identity')
 original=args.app.expanduser().resolve();validate_bundle(original,product)
 notices=(args.notices.expanduser().resolve() if args.notices else original.parent/'Licenses')
 if not notices.is_dir() or not any(f.is_file() and f.name.startswith('LICENSE') for f in notices.iterdir()):p.error('Matching license notices are required; pass --notices with the Licenses folder from package.py')
 if any(f.is_symlink() for f in notices.rglob('*')):p.error('License notices must not contain symlinks')
 output=args.output.expanduser().resolve()
 output.mkdir(parents=True,exist_ok=True)
 name=f"{product['id']}-{product['version']}-macos-{args.architecture}-candidate"
 final=output/(name+'.dmg');report=output/(name+'.json')
 if final.exists() or report.exists():p.error('Candidate output already exists; choose a fresh output folder')
 with tempfile.TemporaryDirectory(prefix='pixa-sign-') as td:
  temporary=Path(td);app=temporary/(product['name']+'.app')
  run(['/usr/bin/ditto',original,app]);binary=validate_bundle(app,product)
  archs=set(run(['/usr/bin/lipo','-archs',binary],capture=True).stdout.split())
  expected={'arm64','x86_64'} if args.architecture=='universal' else {args.architecture}
  if archs!=expected:raise ValueError('Executable architectures do not match the requested installer: '+', '.join(sorted(archs)))
  # Sign nested Mach-O files and code bundles inside out before sealing the app.
  nested=sorted([f for f in app.rglob('*') if macho(f) and f!=binary],key=lambda f:len(f.parts),reverse=True)
  bundles=sorted([f for f in app.rglob('*') if f.is_dir() and f.suffix in {'.framework','.bundle','.xpc','.app'}],key=lambda f:len(f.parts),reverse=True)
  for file in nested+bundles+[app]:run(['/usr/bin/codesign','--force','--options','runtime','--timestamp','--sign',args.identity,file])
  run(['/usr/bin/codesign','--verify','--deep','--strict','--verbose=2',app])
  archive=temporary/'notarize.zip'
  run(['/usr/bin/ditto','-c','-k','--keepParent',app,archive])
  result=run(['/usr/bin/xcrun','notarytool','submit',archive,'--keychain-profile',args.profile,'--wait','--output-format','json'],capture=True)
  submission=json.loads(result.stdout)
  if submission.get('status')!='Accepted':raise RuntimeError('Apple did not accept this submission; inspect its notary log before continuing')
  run(['/usr/bin/xcrun','stapler','staple',app]);run(['/usr/bin/xcrun','stapler','validate',app])
  run(['/usr/sbin/spctl','--assess','--type','execute','--verbose=2',app])
  stage=temporary/'image';stage.mkdir();run(['/usr/bin/ditto',app,stage/app.name])
  shutil.copytree(notices,stage/'Licenses')
  (stage/'Applications').symlink_to('/Applications')
  # The Finder Applications alias supports the standard drag-to-Applications install.
  (stage/'Install Pixa.txt').write_text('Drag '+app.name+' to Applications, then open it from Applications.\nThis is a test candidate; notarization does not establish product/release readiness.\n')
  run(['/usr/bin/hdiutil','create','-volname',product['name'],'-srcfolder',stage,'-format','UDZO',temporary/'candidate.dmg'])
  dmg=temporary/'candidate.dmg'
  run(['/usr/bin/codesign','--force','--timestamp','--sign',args.identity,dmg])
  dmg_result=json.loads(run(['/usr/bin/xcrun','notarytool','submit',dmg,'--keychain-profile',args.profile,'--wait','--output-format','json'],capture=True).stdout)
  if dmg_result.get('status')!='Accepted':raise RuntimeError('Apple did not accept the DMG; no final installer was written')
  run(['/usr/bin/xcrun','stapler','staple',dmg]);run(['/usr/bin/xcrun','stapler','validate',dmg])
  run(['/usr/bin/codesign','--verify','--strict','--verbose=2',dmg])
  shutil.copy2(dmg,final)
  report.write_text(json.dumps({'product':product['id'],'version':product['version'],'bundle_id':product['bundle_id'],'architectures':sorted(archs),'app_submission_id':submission.get('id'),'dmg_submission_id':dmg_result.get('id'),'notarized':True,'sha256':hashlib.sha256(final.read_bytes()).hexdigest(),'file':final.name,'release_ready':False,'published':False,'note':'Signing candidate only. Native editor/account/update acceptance and release approval remain separate.'},indent=2)+'\n')
 print('Notarized test candidate: '+str(final))
 print('No public release, website, subscription, or private-key export was performed.')
if __name__=='__main__':
 try:main()
 except (ValueError,OSError,subprocess.CalledProcessError,json.JSONDecodeError,RuntimeError) as error:
  print('Candidate preparation stopped: '+str(error),file=sys.stderr);raise SystemExit(1)
