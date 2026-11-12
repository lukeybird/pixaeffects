#!/usr/bin/env python3
"""Run real headless egui help interactions for each prepared suite guide.

Tests the shared help UI, not editor integration, installed layout or tool coverage.
No app binary, signing key, Apple service or release operation is involved.
"""
import argparse
import json
from pathlib import Path
import subprocess
import os
import shutil
from manuals import guide

ROOT = Path(__file__).resolve().parent
EGUI_REV = '6d518fc799681571963842f95b55a6b68431b15a'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--app', default='all')
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Use a fresh test output directory')
    apps = json.loads((ROOT / 'catalog.json').read_text())['apps']
    apps = [app for app in apps if args.app in {'all', app['id']}]
    if not apps:
        parser.error('Unknown product')
    output = args.output.resolve()
    (output / 'src').mkdir(parents=True)
    cargo = f'''[workspace]
[package]
name = "pixa-help-ui-acceptance"
version = "0.1.0"
edition = "2024"
publish = false
[dependencies]
egui = {{ git = "https://github.com/storytold/egui.git", rev = "{EGUI_REV}", features = ["default_fonts"] }}
egui_kittest = {{ git = "https://github.com/storytold/egui.git", rev = "{EGUI_REV}" }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
'''
    (output / 'Cargo.toml').write_text(cargo)
    fonts = list(args.sources.resolve().glob('*/assets/fonts/SourceSans3-Regular.ttf'))
    if not fonts:
        raise ValueError('Prepare source fonts before running native help tests')
    shutil.copy2(fonts[0], output / 'src/help-font.ttf')

    # The exact production guide parser is tested without pulling OS vault/network
    # code into a headless help-only crate. No production dependency is changed.
    (output / 'src/help.rs').write_text((ROOT / 'runtime/src/help.rs').read_text())
    lib = 'extern crate self as eframe;\nextern crate self as pixa_runtime;\npub use egui;\npub mod help;\n'
    counts = {}
    for app in apps:
        source = args.sources.resolve() / app['source']
        data = guide(app, source)
        if not data['tools']:
            raise ValueError(app['id'] + ' has no tool/control explanations')
        directory = output / 'src' / app['id']
        directory.mkdir()
        (directory / 'pixa-guide.json').write_text(json.dumps(data))
        text = (ROOT / 'templates/help_ui.rs').read_text().replace('@ID@', app['id']).replace('@NAME@', app['name'])
        if app['id'] == 'pixaanything':
            text = text.replace('egui::Id::unique(', 'egui::Id::new(')
        text += '\n#[cfg(test)]\nmod acceptance {\n' + (ROOT / 'help-ui-tests.rs').read_text() + '\n}\n'
        (directory / 'mod.rs').write_text(text)
        lib += 'pub mod ' + app['id'] + ';\n'
        counts[app['id']] = len(data['tools'])
    (output / 'src/lib.rs').write_text(lib)
    environment = os.environ.copy()
    command = ['cargo', '+1.95.0', 'test', '--manifest-path', str(output / 'Cargo.toml'), '--offline']
    result = subprocess.run(command, env=environment)
    (output / 'result.json').write_text(json.dumps({'products': counts, 'command': command, 'passed': result.returncode == 0, 'scope': 'Shared offline help UI and configured guide navigation. Does not prove exhaustive tool inventory, full editor integration or installed Mac acceptance.'}, indent=2) + '\n')
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
