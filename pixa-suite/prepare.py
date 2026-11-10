#!/usr/bin/env python3
"""Reproducible Pixa source preparation. No release, billing or credential operations."""
from pathlib import Path
import argparse, hashlib, json, os, re, subprocess, tomllib
from PIL import Image, ImageDraw
ROOT = Path(__file__).resolve().parent
CATALOG = json.loads((ROOT/'catalog.json').read_text(encoding="utf-8"))['apps']
SUFFIXES = {'.rs','.toml','.plist','.xml','.desktop','.in','.html','.tsv','.rc','.wxs','.wxl'}

def run(args, cwd=None):
    return subprocess.run(args, cwd=cwd, check=True, text=True, capture_output=True, encoding="utf-8").stdout

def matching_paren(s, start):
    """Balance a Rust expression, skipping comments and ordinary/raw string literals."""
    i=start; depth=0
    while i<len(s):
        if s.startswith('//',i):
            j=s.find('\n',i); i=len(s) if j<0 else j; continue
        if s.startswith('/*',i):
            j=s.find('*/',i+2)
            if j<0: raise ValueError('Unclosed comment')
            i=j+2;continue
        raw=re.match(r'r(#{0,16})"',s[i:])
        if raw:
            end='"'+raw[1];j=s.find(end,i+len(raw[0]))
            if j<0:raise ValueError('Unclosed raw string')
            i=j+len(end);continue
        if s[i]=='"':
            i+=1
            while i<len(s):
                if s[i]=='\\':i+=2;continue
                if s[i]=='"':i+=1;break
                i+=1
            continue
        if s[i]=='(':depth+=1
        elif s[i]==')':
            depth-=1
            if depth==0:return i
        i+=1
    raise ValueError('Unbalanced Rust expression')

def promotional_labels(line):
    for old,new in [('"Discord"','"Account"'),('"GitHub"','"Help"'),('"Join our Discord"','"Your account"'),('"Join the Discord"','"Your account"'),('"Source code on GitHub"','"Product help"'),('"GitHub repository"','"Product help"')]:
        line=line.replace(old,new)
    return line.replace('getartcraft.com','aiwebdesignfirm.com').replace('discord.gg/artcraft','aiwebdesignfirm.com/settings')

def insert_shell_module(s):
    pos=re.search(r'(?m)^(?:use |mod |const |fn |pub |struct |enum |type |extern crate)',s)
    if not pos:raise RuntimeError('No module insertion point')
    outer=re.search(r'(?m)^#\[',s)
    insertion=min(pos.start(),outer.start()) if outer else pos.start()
    return s[:insertion]+'mod pixa_shell;\n'+s[insertion:]

def icon(app,size):
    im=Image.new('RGBA',(size,size),(18,24,35,255));d=ImageDraw.Draw(im)
    # Original Pixa family pixel-P, with a distinct product accent and a small index marker.
    cells=[(0,0),(1,0),(2,0),(0,1),(2,1),(0,2),(1,2),(2,2),(0,3),(0,4)]
    u=size/8
    for x,y in cells:d.rounded_rectangle((int((x+2)*u),int((y+1.5)*u),int((x+2.86)*u),int((y+2.36)*u)),radius=max(1,int(u*.12)),fill=app['color'])
    index=CATALOG.index(app)+1
    for b in range(4):
        if index & (1<<b):d.rectangle((int((b+2)*u),int(7*u),int((b+2.55)*u),int(7.25*u)),fill=app['color'])
    return im

def brand_assets(root,app):
    changed=[]
    for folder in ['assets/app-icon','docs/brand']:
        for p in (root/folder).rglob('*'):
            if not p.is_file() or p.suffix.lower() not in {'.png','.ico','.icns','.svg'}:continue
            if p.suffix.lower()=='.svg':
                im=icon(app,128)
                # SVG matches original pixel geometry; no inherited trademark artwork.
                cells=[(0,0),(1,0),(2,0),(0,1),(2,1),(0,2),(1,2),(2,2),(0,3),(0,4)]
                p.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 80 80"><rect width="80" height="80" rx="16" fill="#121823"/>'+''.join(f'<rect x="{(x+2)*10}" y="{(y+1.5)*10}" width="8.6" height="8.6" rx="1.2" fill="{app["color"]}"/>' for x,y in cells)+'</svg>\n', encoding="utf-8")
            else:
                try:
                    with Image.open(p) as old:size=old.width
                except (OSError,ValueError):size=1024
                size=max(16,min(1024,size)); im=icon(app,size)
                if p.suffix.lower()=='.icns':icon(app,1024).save(p,format='ICNS')
                elif p.suffix.lower()=='.ico':icon(app,256).save(p,format='ICO')
                else:im.save(p,format='PNG')
            rel=p.relative_to(root).as_posix();changed.append(rel)
            Path(str(p)+'.attribution').write_text(f'asset: {p.name}\ntitle: {app["name"]} original pixel mark\nauthor: AIWebDesignFirm\nsource: original work\nlicense: MIT OR Apache-2.0\nlicense-file: LICENSE-MIT\nadded: 2026-11-09 by AIWebDesignFirm\nnotes: Procedural Pixa family mark, not upstream trademark artwork.\n', encoding="utf-8")
    return changed

def prepare(app, sources):
    root=sources/app['source']
    if not root.exists():
        run(['git','init',str(root)])
        run(['git','remote','add','origin','https://github.com/'+app['upstream']+'.git'],root)
        run(['git','fetch','--depth','1','origin',app['revision']],root)
        run(['git','checkout','--detach',app['revision']],root)
    revision=run(['git','rev-parse','HEAD'],root).strip()
    if revision!=app['revision']:raise RuntimeError(f'{app["id"]}: unexpected source revision')
    stamp=root/'.pixa-prepared.json'
    if stamp.exists():raise RuntimeError(f'{app["id"]}: already prepared; use a clean checkout')
    if run(['git','status','--porcelain'],root).strip():raise RuntimeError('Refusing to overwrite local source edits')
    changed=[]
    for folder in ['apps','crates','packaging','assets']:
        for p in (root/folder).rglob('*'):
            if not p.is_file() or p.suffix not in SUFFIXES:continue
            if any(t in p.name.lower() for t in ['license','notice','attribution','contributors']) or 'credits' in p.parts:continue
            try:s=p.read_text(encoding="utf-8")
            except UnicodeDecodeError:continue
            old=s
            # Keep copyright/license lines truthful; internal crate and file-format identifiers stay intact.
            lines=[]
            for line in s.splitlines(keepends=True):
                if 'copyright' not in line.lower():
                    line=line.replace(app['upstream_name'],app['name']).replace('ArtCraft','AIWebDesignFirm')
                    line=line.replace('ai.storyteller.'+app['source'],app['bundle_id'])
                    line=line.replace('https://discord.gg/artcraft','https://aiwebdesignfirm.com/settings')
                    line=line.replace('https://github.com/'+app['upstream']+'/issues','https://aiwebdesignfirm.com').replace('https://github.com/'+app['upstream'],'https://aiwebdesignfirm.com')
                    line=line.replace('Join the AIWebDesignFirm Discord','Manage your account').replace('Join the AIWebDesignFirm community on Discord','Manage your account')
                    line=line.replace('.join(\"'+app['source']+'\")','.join(\"'+app['id']+'\")')
                    if 'i18n' not in str(p) and p.suffix=='.rs':line=promotional_labels(line)
                    line=line.replace('https://getartcraft.com/apps/'+app['source'],'https://aiwebdesignfirm.com')
                    line=line.replace('https://getartcraft.com','https://aiwebdesignfirm.com')
                lines.append(line)
            s=''.join(lines)
            # Never let a branded development editor download an upstream editor as its update.
            if p.name in {'updates_ui.rs','updates.rs'}:
                marker='Updater::new('
                pos=0
                while (start:=s.find(marker,pos))>=0:
                    end=matching_paren(s,start+len('Updater::new'))
                    guard='.with_source(std::sync::Arc::new(|| Err("Pixa updates are not published for this development build.".to_owned())))'
                    s=s[:end+1]+guard+s[end+1:];pos=end+1+len(guard)
            if p.name=='lib.rs' and 'pub const UPDATE_APP' in s:
                a=s.index('pub const UPDATE_APP');b=s.index('\n};',a)+3
                block=s[a:b]
                block=re.sub(r'\.\.craft_update::suite::[A-Z_]+',f'..craft_update::app::App::new("{app["id"]}", "{app["name"]}", "{app["tagline"]}", "lukeybird/{app["id"]}")',block)
                block=block.replace('"storytold/'+app['source']+'"','"lukeybird/'+app['id']+'"')
                block=block.replace('    accent:', '    developer: "AIWebDesignFirm",\n    accent:')
                s=s[:a]+block+s[b:]
            if s!=old:p.write_text(s, encoding="utf-8");changed.append(p.relative_to(root).as_posix())
    cargo=root/'Cargo.toml';s=cargo.read_text(encoding="utf-8");s,n=re.subn(r'(?m)^version = "[^"]+"',f'version = "{app["version"]}"',s,count=1)
    if n!=1:raise RuntimeError('Workspace version not found')
    cargo.write_text(s, encoding="utf-8")
    upstream_manifest=tomllib.loads(run(['git','show','HEAD:Cargo.toml'],root))
    original_version=upstream_manifest['workspace']['package']['version']
    # Path dependencies with explicit requirements must follow the local workspace version.
    for manifest in root.rglob('Cargo.toml'):
        text=manifest.read_text(encoding="utf-8");lines=[]
        for line in text.splitlines(keepends=True):
            if 'path =' in line and 'version = "'+original_version+'"' in line:
                line=line.replace('version = "'+original_version+'"','version = "'+app['version']+'"')
            lines.append(line)
        updated=''.join(lines)
        if updated!=text:manifest.write_text(updated, encoding="utf-8")
    main=root/'apps'/app['source']/'src/main.rs';s=main.read_text(encoding="utf-8")
    marker='Ok(Box::new('
    starts=[m.start() for m in re.finditer(re.escape(marker),s)]
    if len(starts)!=1:raise RuntimeError(f'Expected one desktop creation expression, got {len(starts)}')
    start=starts[0]+len('Ok(');end=matching_paren(s,start+len('Box::new'))
    s=s[:start]+'pixa_shell::wrap('+s[start:end+1]+')'+s[end+1:]
    # Keep outer cfg attributes attached to their original modules.
    main.write_text(insert_shell_module(s), encoding="utf-8")
    from integrations import install
    install(root,app)
    # Source references use the Pixa bundle ID; keep physical asset names in sync.
    for p in list((root/'assets').rglob('*')) + list((root/'packaging').rglob('*')):
        if p.is_file() and 'ai.storyteller.'+app['source'] in p.name:
            p.rename(p.with_name(p.name.replace('ai.storyteller.'+app['source'],app['bundle_id'])))
    assets=brand_assets(root,app)
    attribution=root/'ATTRIBUTION.md'
    with attribution.open('a') as f:
        f.write('\n## Pixa product overlay\nOriginal procedural pixel marks by AIWebDesignFirm, MIT OR Apache-2.0. Generated by pixa-suite/prepare.py.\n')
        for p in assets:f.write(f'- `{p}` — original Pixa pixel mark; AIWebDesignFirm; MIT OR Apache-2.0.\n')
    stamp.write_text(json.dumps({'app':app,'source_changes':changed,'assets':assets,'release_ready':False},indent=2)+'\n', encoding="utf-8")
    print(app['name']+': source prepared; unpublished',flush=True)

def main():
    parser=argparse.ArgumentParser();parser.add_argument('app',nargs='?',default='all');parser.add_argument('--sources',type=Path,default=ROOT/'sources');args=parser.parse_args()
    selected=[a for a in CATALOG if args.app in ('all',a['id'])]
    if not selected:parser.error('Unknown app')
    args.sources.mkdir(parents=True,exist_ok=True)
    for app in selected:prepare(app,args.sources)
if __name__=='__main__':main()
