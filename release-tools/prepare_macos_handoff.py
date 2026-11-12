#!/usr/bin/env python3
"""Stage a matching unsigned suite artifact with notices and privacy metadata.

Never sign, notarize, publish, change the original artifact or approve native QA.
Requires candidate.json from the exact downloaded Mac workflow artifact.
"""
import argparse
import hashlib
import json
from pathlib import Path
import plistlib
import shutil
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'pixa-suite'))
from macos_candidate import validate_bundle


def stage(product, app, source, receipt, output):
    if output.exists():
        raise ValueError('Use a fresh output directory; existing work is preserved')
    prepared = json.loads((source / '.pixa-prepared.json').read_text())
    if prepared.get('app') != product:
        raise ValueError('Prepared source must match the current complete catalog entry')
    data = json.loads(receipt.read_text())
    for field, expected in [('product', product['id']), ('version', product['version']), ('bundle_id', product['bundle_id']), ('revision', product['revision']), ('access_mode', 'subscription')]:
        if data.get(field) != expected:
            raise ValueError('Artifact receipt mismatch: ' + field)
    info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
    executable = info.get('CFBundleExecutable')
    if not isinstance(executable, str) or executable in {'', '.', '..'} or Path(executable).name != executable or '\\' in executable:
        raise ValueError('Invalid artifact executable path')
    binary = app / 'Contents/MacOS' / executable
    if binary.is_symlink() or not binary.is_file():
        raise ValueError('Artifact executable is missing or a symlink')
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    if data.get('sha256') != digest or set(data.get('architectures', [])) != {'arm64', 'x86_64'}:
        raise ValueError('Artifact binary hash or claimed architectures do not match')
    if data.get('signed') is not False or data.get('published') is not False:
        raise ValueError('Expected an unsigned unpublished CI candidate')
    for directory in [app, source]:
        for file in directory.rglob('*'):
            if file.is_symlink() and not file.resolve().is_relative_to(directory.resolve()):
                raise ValueError('Source/artifact contains an escaping symlink')
    # Validate identity before copying, allowing only the missing purpose text
    # that this staging step deliberately supplies from the matching source.
    if info.get('CFBundleIdentifier') != product['bundle_id'] or info.get('CFBundleShortVersionString') != product['version']:
        raise ValueError('Artifact plist identity/version mismatch')
    privacy = {}
    if product['id'] in {'pixafilm', 'pixasound'}:
        template = plistlib.loads((source / 'packaging/macos/Info.plist.in').read_bytes())
        purpose = template.get('NSMicrophoneUsageDescription')
        if not isinstance(purpose, str) or not purpose.strip():
            raise ValueError('Matching source has no microphone purpose')
        privacy['NSMicrophoneUsageDescription'] = purpose
    notices = [file for file in source.iterdir() if file.is_file() and (file.name.startswith(('LICENSE', 'NOTICE')) or file.name in {'ASSETS.md', 'ATTRIBUTION.md'})]
    for base in ['assets', 'docs/brand']:
        notices.extend(file for file in (source / base).rglob('*') if file.is_file() and any(token in file.name.lower() for token in ['license', 'ofl-', 'attribution']))
    if not any(file.name.startswith('LICENSE') for file in notices):
        raise ValueError('Matching source license notices are missing')
    if any(file.is_symlink() for file in notices):
        raise ValueError('Notice files must not be symlinks')
    output.mkdir(parents=True)
    target = output / (product['name'] + '.app')
    shutil.copytree(app, target, symlinks=True)
    info.update(privacy)
    (target / 'Contents/Info.plist').write_bytes(plistlib.dumps(info))
    validate_bundle(target, product)
    for file in notices:
        destination = output / 'Licenses' / file.relative_to(source)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(file, destination)
    from manuals import markdown
    (output / (product['name'] + '-Manual.md')).write_text(markdown(product, source), encoding='utf-8')
    shutil.copy2(receipt, output / 'unsigned-build-receipt.json')
    report = {'product': product['id'], 'version': product['version'], 'upstream_revision': product['revision'], 'binary_sha256': digest, 'notice_files': len(notices), 'metadata_supplied': sorted(privacy), 'staged': True, 'signed': False, 'notarized': False, 'native_acceptance': 'pending', 'architectures': 'receipt claims universal; verify actual lipo output on Mac', 'ready_for_notarization': False, 'note': 'CI receipt is not independent source provenance. Confirm the run source SHA and artifact digest before signing. This tool only stages an unsigned copy.'}
    (output / 'handoff.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('product')
    parser.add_argument('--app', required=True, type=Path)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--receipt', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    products = json.loads((ROOT / 'pixa-suite/catalog.json').read_text())['apps']
    product = next((item for item in products if item['id'] == args.product), None)
    if product is None:
        parser.error('Unknown suite product; Shop/Effects use their own process')
    print(json.dumps(stage(product, args.app.resolve(), args.source.resolve(), args.receipt.resolve(), args.output.resolve()), indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, KeyError) as error:
        print('Staging stopped: ' + str(error), file=sys.stderr)
        raise SystemExit(1)
