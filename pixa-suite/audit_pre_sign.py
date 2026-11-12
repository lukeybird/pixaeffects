#!/usr/bin/env python3
"""Reproducible source preflight; never infers native acceptance or signing readiness."""
import argparse,hashlib,json
from pathlib import Path
from manuals import guide
ROOT=Path(__file__).resolve().parent
def audit(app,source):
 failures=[]
 if not (source/'.pixa-prepared.json').is_file():failures.append('source not prepared')
 for file in (source/'packaging').rglob('*'):
  if file.suffix in {'.yml','.yaml'}:
   text=file.read_text()
   if 'ai.storyteller.'+app['source'] in text:failures.append(str(file.relative_to(source))+': stale bundle ID')
  if file.name.endswith('.metainfo.xml.in'):
   text=file.read_text()
   if '<developer id="ai.storyteller">' in text or '<screenshots>' in text:failures.append(str(file.relative_to(source))+': stale publisher/screenshots')
 licenses=[f.name for f in source.glob('LICENSE*') if f.is_file()]
 if not licenses:failures.append('license missing')
 main=list((source/'apps').rglob('pixa-guide.json'))
 if len(main)!=1:failures.append('embedded manual missing or ambiguous')
 data=json.loads(main[0].read_text()) if len(main)==1 else {}
 expected=guide(app,source)
 if data!=expected:failures.append('embedded manual differs from current generator')
 return {'product':app['id'],'upstream_revision':app['revision'],'checks_passed':not failures,'failures':failures,'licenses_retained':licenses,'embedded_starter_chapters':len(data.get('chapters',[])),'embedded_tour_steps':len(data.get('tour',[])),'tool_reference_chapters':len(data.get('tools',[])),'exhaustive_manual_verified':False,'graphical_acceptance':'not-run','signed_updater_install_restart':'implemented-macos-unverified','ready_for_signing':False}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--sources',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 apps=json.loads((ROOT/'catalog.json').read_text())['apps'];results=[audit(x,a.sources/x['source']) for x in apps]
 a.output.write_text(json.dumps({'schema_version':1,'scope':'Source preflight only. Historical CI is recorded separately; it does not verify these new changes.','apps':results},indent=2)+'\n')
 print(f'{sum(x["checks_passed"] for x in results)}/{len(results)} source preflights passed; 0 signing approvals')
 raise SystemExit(any(not x['checks_passed'] for x in results))
