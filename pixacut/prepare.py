#!/usr/bin/env python3
"""Reconstruct the exact PixaCut prototype without overwriting an existing checkout."""
from pathlib import Path
import argparse,hashlib,json,subprocess,sys
ROOT=Path(__file__).resolve().parent

def run(*args,cwd=None):subprocess.run(list(map(str,args)),cwd=cwd,check=True)
def verify(destination,manifest):
    for name,digest in manifest['changed_files'].items():
        relative=Path(name)
        if relative.is_absolute() or '..' in relative.parts:raise ValueError('Unsafe source manifest path')
        path=destination/relative
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest()!=digest:raise ValueError('Reconstructed file differs: '+name)
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--destination',type=Path,default=ROOT/'sources/PixaCut');args=parser.parse_args()
    target=args.destination.expanduser().absolute();manifest=json.loads((ROOT/'SOURCE.json').read_text());patch=ROOT/'branding.patch'
    if target.exists():parser.error('Destination already exists. Choose a new path; no existing files were changed.')
    if hashlib.sha256(patch.read_bytes()).hexdigest()!=manifest['patch_sha256']:parser.error('Branding patch checksum mismatch')
    target.parent.mkdir(parents=True,exist_ok=True)
    run('git','clone','--no-checkout','--depth','1',manifest['upstream'],target)
    run('git','fetch','--depth','1','origin',manifest['revision'],cwd=target)
    run('git','-c','core.autocrlf=false','checkout','--detach',manifest['revision'],cwd=target)
    run('git','apply','--check',patch,cwd=target)
    run('git','apply',patch,cwd=target)
    verify(target,manifest)
    print('Prepared PixaCut at '+str(target))
    print('Verified '+str(len(manifest['changed_files']))+' changed files. Read PIXACUT_STATUS.json before building.')
    print('This is source, not a signed Mac app. Native tests/build/signing remain pending.')
if __name__=='__main__':
    try:main()
    except (OSError,ValueError,subprocess.CalledProcessError) as error:raise SystemExit(str(error))
