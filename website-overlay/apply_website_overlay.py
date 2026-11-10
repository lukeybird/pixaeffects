#!/usr/bin/env python3
"""Apply only the Pixa marketing patch to a complete local website; never deploy or edit billing."""
from pathlib import Path
import argparse, hashlib, json, shutil, tempfile
ROOT=Path(__file__).resolve().parent
SOURCE_FOLDERS=('app','components','lib','public')
REQUIRED=('package.json','app/layout.tsx','app/api/billing/route.ts','app/api/pixashop/device/route.ts','app/pixashop/page.tsx','lib/pixashop.ts','app/builder/page.tsx')

def plan(target):
 target=target.resolve()
 if target==ROOT or ROOT.is_relative_to(target):raise ValueError('Choose the full website checkout, not the patch repository or its parent')
 missing=[p for p in REQUIRED if not (target/p).is_file()]
 if missing:raise ValueError('Incomplete website source; missing: '+', '.join(missing))
 package=json.loads((target/'package.json').read_text())
 if 'next' not in package.get('dependencies',{}):raise ValueError('Target is not the Next.js website')
 files=[]
 for folder in SOURCE_FOLDERS:
  for source in (ROOT/folder).rglob('*'):
   if source.is_file():
    relative=source.relative_to(ROOT)
    destination=target/relative
    if not destination.resolve().is_relative_to(target):raise ValueError('Destination follows a symlink outside the checkout')
    files.append((relative,source,destination))
 return files

def apply(target):
 files=plan(target)
 # Backup all destinations before the first write; keep backups outside Git/public.
 backup=Path(tempfile.mkdtemp(prefix='pixa-site-backup-'))
 record=[]
 for relative,source,destination in files:
  previous=backup/relative
  existed=destination.is_file()
  if existed:previous.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(destination,previous)
  record.append({'path':relative.as_posix(),'existed':existed,'new_sha256':hashlib.sha256(source.read_bytes()).hexdigest()})
 (backup/'manifest.json').write_text(json.dumps({'target':str(target.resolve()),'files':record},indent=2))
 for relative,source,destination in files:
  destination.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,destination)
 return backup

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('website',type=Path);parser.add_argument('--apply',action='store_true');args=parser.parse_args()
 try:
  planned=plan(args.website)
  print('Marketing files: '+str(len(planned)))
  for relative,_,_ in planned:print(relative)
  if args.apply:print('Previous files backed up at: '+str(apply(args.website)))
  else:print('Read-only check. Add --apply after checking the full local production source and preserving existing work.')
 except (ValueError,OSError,json.JSONDecodeError) as error:parser.exit(1,str(error)+'\n')
