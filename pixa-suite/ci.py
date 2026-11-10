#!/usr/bin/env python3
"""One isolated product check on a hosted runner; never creates releases or contacts the account API."""
from pathlib import Path
import argparse, json, os, subprocess, sys, tempfile, venv
ROOT=Path(__file__).resolve().parent

def command(app_id):
 apps=json.loads((ROOT/'catalog.json').read_text(encoding="utf-8"))['apps']
 return next((a for a in apps if a['id']==app_id),None)

def run(args, env=None):
 subprocess.run(args,cwd=ROOT.parent,env=env,check=True,stdin=subprocess.DEVNULL)

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('app');args=p.parse_args();app=command(args.app)
 if app is None:p.error('Unknown product')
 with tempfile.TemporaryDirectory(prefix='pixa-ci-') as temporary:
  environment=Path(temporary)/'python';venv.EnvBuilder(with_pip=True).create(environment)
  python=environment/('Scripts/python.exe' if os.name=='nt' else 'bin/python')
  run([str(python),'-m','pip','install','--disable-pip-version-check','-r',str(ROOT/'requirements.txt')])
  run(['rustup','toolchain','install',app['rust_toolchain'],'--profile','minimal','--component','rustfmt'])
  run([str(python),'-m','unittest','discover','-s',str(ROOT),'-p','test_*.py'])
  run([str(python),str(ROOT/'prepare.py'),app['id']])
  run([str(python),str(ROOT/'check.py'),'--native','--app',app['id']])
  from integrations import PILOTS
  if app['id'] in PILOTS:
   restricted=os.environ.copy();restricted['PIXA_ACCESS_MODE']='subscription'
   run([str(python),str(ROOT/'check.py'),'--native','--app',app['id']],env=restricted)
   fixture=os.environ.copy();fixture['CARGO_TARGET_DIR']=str(ROOT/'target');fixture['CARGO_PROFILE_DEV_DEBUG']='0'
   run(['cargo','+'+app['rust_toolchain'],'run','--manifest-path',str(ROOT/'sources'/app['source']/'Cargo.toml'),'-p',app['source']+'-engine','--example','pixa_recovery_check'],env=fixture)
  if app['id']=='pixafilm':
   run(['cargo','+1.95.0','test','--manifest-path',str(ROOT/'runtime/Cargo.toml')])

