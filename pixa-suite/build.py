#!/usr/bin/env python3
"""Build one prepared native app and write a hash-bound build receipt."""
from pathlib import Path
import argparse, hashlib, json, os, subprocess, sys
ROOT=Path(__file__).resolve().parent
def source_digest(source):
 digest=hashlib.sha256()
 paths=subprocess.check_output(['git','ls-files','-c','-o','--exclude-standard','-z'],cwd=source).split(b'\0')
 for raw in sorted(set(paths)):
  if not raw:continue
  rel=os.fsdecode(raw);file=source/rel
  if file.is_file():digest.update(raw+b'\0'+hashlib.sha256(file.read_bytes()).digest())
 runtime=ROOT/'runtime'
 runtime_inputs=[runtime/'Cargo.toml']+list((runtime/'src').rglob('*.rs'))
 for file in sorted(runtime_inputs):
  if file.is_file():digest.update(('pixa-runtime/'+file.relative_to(runtime).as_posix()).encode()+b'\0'+hashlib.sha256(file.read_bytes()).digest())
 return digest.hexdigest()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('app');p.add_argument('--release',action='store_true');args=p.parse_args()
 apps=json.loads((ROOT/'catalog.json').read_text())['apps'];app=next((a for a in apps if a['id']==args.app),None)
 if not app:p.error('Unknown app')
 source=ROOT/'sources'/app['source']
 if not (source/'.pixa-prepared.json').exists():p.error('Run prepare.py first')
 logs=ROOT/'build-logs';logs.mkdir(exist_ok=True)
 receipt=logs/(app['id']+'-release.json')
 if receipt.exists():receipt.unlink()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'target')
 command=['cargo','+'+app['rust_toolchain'],'build','-p',app['package']]+(['--release'] if args.release else [])
 with (logs/(app['id']+'-build.log')).open('w') as log:
  result=subprocess.run(command,cwd=source,env=env,stdout=log,stderr=subprocess.STDOUT)
 if result.returncode:raise SystemExit(result.returncode)
 if args.release:
  binary=ROOT/'target/release'/(app['package']+('.exe' if sys.platform=='win32' else ''))
  receipt.write_text(json.dumps({'app':app['id'],'revision':app['revision'],'source_digest':source_digest(source),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'platform':sys.platform,'signed':False,'release_ready':False},indent=2)+'\n')
 print('Built '+app['name']+'; platform QA and release gates still apply.')
