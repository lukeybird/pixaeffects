#!/usr/bin/env python3
"""Build an unsigned universal launcher candidate. Does not submit to Apple."""
import argparse, hashlib, json, os, plistlib, shutil, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent

def run(*args):
    subprocess.run(list(map(str,args)),check=True)

def package(out, binaries):
    out=out.resolve()
    if out.exists():
        raise RuntimeError('Output already exists; choose a new directory. Nothing was removed.')
    out.mkdir(parents=True)
    app=out/'Pixa Launcher.app';mac=app/'Contents/MacOS';res=app/'Contents/Resources';mac.mkdir(parents=True);res.mkdir()
    exe=mac/'pixa-launcher'
    run('/usr/bin/lipo','-create',*binaries,'-output',exe)
    exe.chmod(0o755)
    with (app/'Contents/Info.plist').open('wb') as f:
        plistlib.dump({'CFBundleIdentifier':'com.aiwebdesignfirm.pixalauncher','CFBundleName':'Pixa Launcher','CFBundleDisplayName':'Pixa Launcher','CFBundleExecutable':'pixa-launcher','CFBundleShortVersionString':'0.2.0','CFBundleVersion':'2','CFBundlePackageType':'APPL','LSMinimumSystemVersion':'11.0','NSHighResolutionCapable':True,'NSPrincipalClass':'NSApplication','CFBundleIconFile':'PixaLauncher.icns'},f)
    # Original pixel P mark, generated through standard library PNG; no image dependency.
    import struct,zlib
    iconset=out/'PixaLauncher.iconset';iconset.mkdir()
    def png(size):
        pixels=[];glyph=['11110','10001','10001','11110','10000','10000','10000']
        for y in range(size):
            row=bytearray()
            for x in range(size):
                col=int((x/size-.25)/.1);line=int((y/size-.15)/.1)
                on=.25<=x/size<.75 and .15<=y/size<.85 and 0<=line<7 and 0<=col<5 and glyph[line][col]=='1'
                row.extend((114,212,237,255) if on else (17,25,38,255))
            pixels.append(b'\0'+row)
        def chunk(t,b):return struct.pack('>I',len(b))+t+b+struct.pack('>I',zlib.crc32(t+b)&0xffffffff)
        return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',size,size,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(pixels)))+chunk(b'IEND',b'')
    for size in [16,32,128,256,512]:
        (iconset/f'icon_{size}x{size}.png').write_bytes(png(size))
        (iconset/f'icon_{size}x{size}@2x.png').write_bytes(png(size*2))
    run('/usr/bin/iconutil','-c','icns',iconset,'-o',res/'PixaLauncher.icns')
    for name in ['README.md','ATTRIBUTION.md','THIRD_PARTY_LICENSES.txt','catalog.json']:
        shutil.copyfile(ROOT/name,res/name)
    # Locally runnable ad-hoc signature only. Developer ID signing remains separate.
    run('/usr/bin/codesign','--force','--sign','-',app)
    run('/usr/bin/codesign','--verify','--deep','--strict',app)
    sha=hashlib.sha256(exe.read_bytes()).hexdigest()
    (out/'CANDIDATE.json').write_text(json.dumps({'product':'Pixa Launcher','version':'0.2.0','architecture':'universal','executable_sha256':sha,'apple_signed':False,'notarized':False,'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_dirty':bool(subprocess.check_output(['git','status','--porcelain','--','pixa-launcher'],cwd=ROOT.parent,text=True).strip()),'next':'Run Mac acceptance before Developer ID signing/notarization.'},indent=2)+'\n')
    print(f'Unsigned universal candidate: {app}')
    return app

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True);parser.add_argument('--arm64',type=Path);parser.add_argument('--intel',type=Path);args=parser.parse_args()
    if sys.platform!='darwin':parser.error('Run on a Mac with Rust and Xcode command-line tools.')
    if bool(args.arm64)!=bool(args.intel):parser.error('Supply both arm64 and Intel binaries, or neither.')
    if args.arm64:
        binaries=[args.arm64.resolve(),args.intel.resolve()]
    else:
        run('rustup','target','add','aarch64-apple-darwin','x86_64-apple-darwin')
        for target in ['aarch64-apple-darwin','x86_64-apple-darwin']:
            run('cargo','build','--locked','--release','--manifest-path',ROOT/'Cargo.toml','--target',target)
        binaries=[ROOT/'target'/target/'release/pixa-launcher' for target in ['aarch64-apple-darwin','x86_64-apple-darwin']]
    package(args.out,binaries)
if __name__=='__main__':
    try:main()
    except (RuntimeError,subprocess.CalledProcessError,OSError) as e:raise SystemExit(str(e))
