#!/usr/bin/env python3
"""Package the compiled editor under PixaEffects's identity; never relabel an upstream binary."""
from pathlib import Path
import os, platform, shutil, subprocess, sys, tarfile, tomllib, zipfile, hashlib, json
ROOT=Path(__file__).resolve().parent.parent

def package():
    version=tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['package']['version']
    if not (ROOT/'pixaeffects/.prepared').exists():
        raise RuntimeError('Run pixaeffects/prepare.py before compiling the editor.')
    out=ROOT/'dist/pixaeffects'; out.mkdir(parents=True,exist_ok=True)
    system=platform.system()
    target=Path(os.environ.get('CARGO_TARGET_DIR',ROOT/'target'))
    stage=out/'stage'
    if stage.exists(): shutil.rmtree(stage)
    stage.mkdir()
    licenses=stage/'Licenses'; licenses.mkdir(exist_ok=True)
    for name in ['LICENSE-MIT','LICENSE-APACHE','NOTICE','ATTRIBUTION.md']:
        shutil.copyfile(ROOT/name,licenses/name)
    # Preserve all bundled font, dictionary and icon license notices.
    for source in (ROOT/'assets').rglob('*'):
        if source.is_file() and any(x in source.name.lower() for x in ['license','ofl-','attribution']):
            dest=licenses/source.relative_to(ROOT/'assets');dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,dest)
    if system=='Windows':
        shutil.copyfile(target/'release/effectcraft.exe',stage/'PixaEffects.exe')
        (stage/'portable.txt').write_text('PixaEffects portable installation\n')
        asset=out/f'pixaeffects-{version}-windows-x64-portable.zip'
        with zipfile.ZipFile(asset,'w',zipfile.ZIP_DEFLATED) as archive:
            for p in stage.rglob('*'):
                if p.is_file():archive.write(p,Path('PixaEffects')/p.relative_to(stage))
    elif system=='Darwin':
        app=stage/'PixaEffects.app'; macos=app/'Contents/MacOS'; resources=app/'Contents/Resources';macos.mkdir(parents=True,exist_ok=True);resources.mkdir(parents=True,exist_ok=True)
        subprocess.run(['lipo','-create',str(target/'aarch64-apple-darwin/release/effectcraft'),str(target/'x86_64-apple-darwin/release/effectcraft'),'-output',str(macos/'pixaeffects')],check=True)
        shutil.copyfile(ROOT/'assets/app-icon/effectcraft.icns',resources/'PixaEffects.icns');shutil.copytree(licenses,resources/'Licenses',dirs_exist_ok=True)
        import plistlib
        with (app/'Contents/Info.plist').open('wb') as f:
            plistlib.dump({'CFBundleName':'PixaEffects','CFBundleDisplayName':'PixaEffects','CFBundleIdentifier':'com.aiwebdesignfirm.pixaeffects','CFBundleExecutable':'pixaeffects','CFBundlePackageType':'APPL','CFBundleShortVersionString':version,'CFBundleVersion':version,'CFBundleIconFile':'PixaEffects','LSMinimumSystemVersion':'11.0','NSHighResolutionCapable':True},f)
        subprocess.run(['codesign','--force','--deep','--sign','-',str(app)],check=True)
        asset=out/f'pixaeffects-{version}-macos-universal.dmg'
        from macos_dmg import build_installer
        build_installer(app, asset)
    else:
        shutil.copyfile(target/'release/effectcraft',stage/'pixaeffects');(stage/'pixaeffects').chmod(0o755)
        asset=out/f'pixaeffects-{version}-linux-x86_64.tar.gz'
        with tarfile.open(asset,'w:gz') as archive:archive.add(stage,arcname='PixaEffects')
    digest=hashlib.sha256(asset.read_bytes()).hexdigest()
    (out/(asset.name+'.sha256')).write_text(f'{digest}  {asset.name}\n')
    print(json.dumps({'asset':str(asset),'sha256':digest,'signed':False,'status':'testing-only'}))
    return asset

if __name__=='__main__':
    try:package()
    except (OSError,RuntimeError,subprocess.CalledProcessError) as error:
        print(f'PixaEffects packaging failed: {error}',file=sys.stderr);sys.exit(1)
