#!/usr/bin/env python3
"""Export the same source-bound offline guides for the static website."""
import argparse,json
from pathlib import Path
from manuals import guide
ROOT=Path(__file__).resolve().parent
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--sources',type=Path,required=True);p.add_argument('--output',type=Path,default=ROOT/'manuals/generated');a=p.parse_args()
 a.output.mkdir(parents=True,exist_ok=True)
 for app in json.loads((ROOT/'catalog.json').read_text())['apps']:
  source=a.sources/app['source'];stamp=source/'.pixa-prepared.json'
  if not stamp.is_file():raise RuntimeError('Prepare the pinned source first: '+app['id'])
  document=guide(app,source)
  if not document['tools']:raise RuntimeError('Missing source-bound control reference: '+app['id'])
  (a.output/(app['id']+'.json')).write_text(json.dumps(document,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
