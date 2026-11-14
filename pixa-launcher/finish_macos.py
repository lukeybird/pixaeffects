#!/usr/bin/env python3
"""Finish an accepted launcher candidate using the publisher Mac's existing Keychain."""
import argparse,hashlib,importlib.util,json,shutil,subprocess,sys,tempfile,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
CHECKS=['library_search_help','install_and_open','cancel_and_retry','running_app_update_refused','previous_app_preserved','documents_survive_update','editor_account_activation']
IDENTITY='Developer ID Application: Luke Barger (945M5X4DY3)'

def acceptance(app,report_path,acceptance_path):
    info_path=app/'Contents/Info.plist'
    import plistlib
    info=plistlib.loads(info_path.read_bytes())
    if info.get('CFBundleIdentifier')!='com.aiwebdesignfirm.pixalauncher' or info.get('CFBundleShortVersionString')!='0.2.0':raise ValueError('Wrong launcher bundle/version')
    name=info.get('CFBundleExecutable')
    if not isinstance(name,str) or Path(name).name!=name or name in {'.','..',''}:raise ValueError('Invalid executable name')
    digest=hashlib.sha256((app/'Contents/MacOS'/name).read_bytes()).hexdigest()
    report=json.loads(report_path.read_text())
    if report.get('executable_sha256')!=digest or report.get('architecture')!='universal' or report.get('source_dirty') is not False:raise ValueError('Candidate receipt does not match a clean universal build')
    if not re.fullmatch(r'[0-9a-f]{40}',str(report.get('source_commit',''))):raise ValueError('Candidate source commit is missing or invalid')
    record=json.loads(acceptance_path.read_text())
    if record.get('executable_sha256')!=digest or record.get('source_commit')!=report.get('source_commit'):raise ValueError('Acceptance record belongs to another build')
    if any(record.get('checks',{}).get(k)!='passed' for k in CHECKS):raise ValueError('Real-Mac acceptance remains pending; do not invent pass results')
    return report

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('app',type=Path);p.add_argument('--receipt',type=Path,required=True);p.add_argument('--acceptance',type=Path,required=True);p.add_argument('--profile',default='pixaWorld');p.add_argument('--output',type=Path,required=True);p.add_argument('--publish',action='store_true',help='Upload the final notarized DMG to GitHub; never changes the website automatically')
    args=p.parse_args()
    if sys.platform!='darwin':p.error('Run on the publisher Mac with its existing private signing identity')
    app=args.app.expanduser().resolve();report=acceptance(app,args.receipt.expanduser(),args.acceptance.expanduser())
    output=args.output.expanduser().resolve()
    if output.exists():p.error('Choose a new output directory; nothing will be overwritten')
    spec=importlib.util.spec_from_file_location('mac_candidate',ROOT.parent/'release-tools/macos_candidate.py');helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
    with tempfile.TemporaryDirectory(prefix='pixa-launcher-release-') as td:
        temp=Path(td);catalog=temp/'catalog';catalog.mkdir();(catalog/'catalog.json').write_text(json.dumps({'apps':[{'id':'pixalauncher','name':'Pixa Launcher','version':'0.2.0','bundle_id':'com.aiwebdesignfirm.pixalauncher'}]}))
        notices=temp/'Licenses';notices.mkdir();shutil.copyfile(ROOT/'THIRD_PARTY_LICENSES.txt',notices/'LICENSE-PixaLauncher-dependencies.txt');shutil.copyfile(ROOT/'ATTRIBUTION.md',notices/'ATTRIBUTION.md')
        helper.ROOT=catalog
        old=sys.argv
        try:
            sys.argv=[str(ROOT.parent/'release-tools/macos_candidate.py'),'pixalauncher',str(app),'--identity',IDENTITY,'--profile',args.profile,'--output',str(output),'--notices',str(notices)]
            helper.main()
        finally:sys.argv=old
    old_dmg=output/'pixalauncher-0.2.0-macos-universal-candidate.dmg';final=output/'pixalauncher-0.2.0-macos-universal.dmg';shutil.copyfile(old_dmg,final)
    signed=json.loads((output/'pixalauncher-0.2.0-macos-universal-candidate.json').read_text())
    if signed.get('notarized') is not True or hashlib.sha256(final.read_bytes()).hexdigest()!=signed.get('sha256'):raise ValueError('Final notarization/hash report mismatch; no release uploaded')
    # Validate the app inside the final DMG, not only the pre-image staging app.
    with tempfile.TemporaryDirectory(prefix='pixa-launcher-final-check-') as td:
        mount=Path(td)/'mounted';mount.mkdir()
        helper.run(['/usr/bin/hdiutil','attach','-readonly','-nobrowse','-quiet','-mountpoint',mount,final])
        try:
            enclosed=mount/'Pixa Launcher.app'
            helper.run(['/usr/bin/codesign','--verify','--deep','--strict','-R','anchor apple generic and identifier "com.aiwebdesignfirm.pixalauncher" and certificate leaf[subject.OU] = "945M5X4DY3"',enclosed])
            helper.run(['/usr/sbin/spctl','--assess','--type','execute',enclosed])
            helper.run(['/usr/bin/xcrun','stapler','validate',enclosed])
            helper.run([enclosed/'Contents/MacOS/pixa-launcher','--version'])
        finally:helper.run(['/usr/bin/hdiutil','detach','-quiet',mount])
    checksum=final.with_suffix('.dmg.sha256');checksum.write_text(signed['sha256']+'  '+final.name+'\n')
    signed.update({'file':final.name,'size':final.stat().st_size,'source_commit':report['source_commit'],'acceptance_recorded':True,'release_ready':True,'published':False});release_report=output/'PIXA_LAUNCHER_NOTARIZED.json';release_report.write_text(json.dumps(signed,indent=2)+'\n')
    if args.publish:
        notes=output/'RELEASE_NOTES.txt';notes.write_text('Pixa Launcher 0.2.0 — Mac universal installer.\n\nDeveloper ID signed for Team 945M5X4DY3 and accepted by Apple notarization. Native Mac acceptance recorded for source '+report['source_commit']+'.\n\nSHA-256: '+signed['sha256']+'\n')
        subprocess.run(['gh','release','create','pixalauncher-v0.2.0',str(final),str(checksum),str(release_report),'--repo','lukeybird/pixaeffects','--target',report['source_commit'],'--title','Pixa Launcher 0.2.0','--notes-file',str(notes),'--latest=false'],check=True)
        signed['published']=True;release_report.write_text(json.dumps(signed,indent=2)+'\n')
        subprocess.run(['gh','release','upload','pixalauncher-v0.2.0',str(release_report),'--clobber','--repo','lukeybird/pixaeffects'],check=True)
        print('Published final launcher release. Website enablement still requires checking the exact public asset URL/hash.')
    else:print('Final notarized launcher: '+str(final)+'\nNot uploaded; use --publish to publish a verified release.')
if __name__=='__main__':
    try:main()
    except (OSError,ValueError,subprocess.CalledProcessError) as e:raise SystemExit(str(e))
