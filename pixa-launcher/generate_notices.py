#!/usr/bin/env python3
"""Collect resolved Mac dependency license files without downloading external artwork."""
import json,subprocess,urllib.request,urllib.error,concurrent.futures,re,hashlib
from urllib.parse import urlparse
from pathlib import Path
ROOT=Path(__file__).resolve().parent

def generate():
    m=json.loads(subprocess.check_output(['cargo','metadata','--manifest-path',str(ROOT/'Cargo.toml'),'--locked','--format-version','1','--filter-platform','aarch64-apple-darwin'],text=True))
    resolved={n['id'] for n in m['resolve']['nodes']}
    cache={}
    def remote(p):
        root=Path(p['manifest_path']).parent
        try:vcs=json.loads((root/'.cargo_vcs_info.json').read_text());ref=vcs['git']['sha1']
        except (OSError,KeyError):ref='master'
        parsed=urlparse(p['repository'] or '');parts=parsed.path.strip('/').split('/')
        if parsed.hostname!='github.com' or len(parts)<2:return []
        base='https://raw.githubusercontent.com/'+parts[0]+'/'+parts[1].removesuffix('.git')+'/'+ref+'/'
        if base in cache:return cache[base]
        candidates=['LICENSE','LICENSE-MIT','LICENSE-APACHE','LICENSE.MIT','LICENSE.APACHE','LICENSE.md','LICENSE.txt','COPYING']
        def fetch(name):
            try:
                with urllib.request.urlopen(base+name,timeout=30) as r:return (base+name,r.read(100000).decode())
            except urllib.error.HTTPError as e:
                if e.code==404:return None
                raise
        with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:result=[x for x in pool.map(fetch,candidates) if x]
        cache[base]=result;return result
    text=['PIXA LAUNCHER — THIRD-PARTY NOTICES\nResolved by Cargo.lock; Mac dependency graph.\n']
    missing=[]
    for p in sorted(m['packages'],key=lambda p:(p['name'],p['version'])):
        if p['id'] not in resolved or p['name']=='pixa-launcher':continue
        text.append('\n'+'='*72+'\n'+p['name']+' '+p['version']+'\nSPDX: '+str(p['license'])+'\nSource: '+str(p['repository'])+'\n')
        root=Path(p['manifest_path']).parent
        files=sorted(f for f in root.rglob('*') if f.is_file() and len(f.relative_to(root).parts)<5 and (f.name.lower().startswith(('license','licence','notice','copyright')) or (f.parent.name.lower() in ['license','licenses','licences']) or f.name in ['OFL.txt','UFL.txt']))
        # Some crates omit workspace-root notices from the crates.io tarball.
        # Retrieve the notices at the crate's exact recorded source revision.
        roots=[f for f in files if f.parent==root]
        if not roots:
            remote_files=remote(p)
            for url,body in remote_files:text.append('\n--- '+url+' ---\n'+body+'\n')
            if not remote_files and not files:
                # These published crates declare SPDX licenses but omit a text file.
                # Include the canonical license verbatim and the declared authors;
                # do not invent an upstream copyright year or a different license.
                license_id='MIT' if p['license']=='MIT' else ('CC0-1.0' if p['license']=='CC0-1.0' else None)
                if license_id:
                    url='https://raw.githubusercontent.com/spdx/license-list-data/v3.26.0/text/'+license_id+'.txt'
                    with urllib.request.urlopen(url,timeout=30) as response:body=response.read().decode()
                    text.append('\nCrate omits a license text; canonical SPDX '+license_id+' text follows. Declared authors: '+', '.join(p['authors'])+'\nSource: '+url+'\n'+body+'\n')
                else:missing.append(p['name'])
        for f in files:
            try:body=f.read_text()
            except UnicodeError:continue
            text.append('\n--- '+str(f.relative_to(root))+' ---\n'+body+'\n')
    if missing:raise RuntimeError('Review packages missing license files: '+', '.join(missing))
    out=ROOT/'THIRD_PARTY_LICENSES.txt';out.write_text(deduplicate(''.join(text)));print(f'Wrote {out.name}')
def deduplicate(text):
    notices={}
    def entry(match):
        body=match.group(2).strip();digest=hashlib.sha256(body.encode()).hexdigest()
        notices[digest]=body
        return match.group(1)+'See full license text '+digest+' in the shared-text appendix below.\n'
    compact=re.sub(r'(\n--- [^\n]+ ---\n)(.*?)(?=\n--- |\n={72}\n|\Z)',entry,text,flags=re.S)
    return compact+'\nFULL LICENSE TEXTS (deduplicated; package references above)\n'+''.join('\nTEXT '+digest+'\n'+body+'\n' for digest,body in notices.items())
if __name__=='__main__':generate()
