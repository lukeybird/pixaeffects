#!/usr/bin/env python3
"""Run the real PixaVector engine workflow fixture; restore the exact prepared source afterward."""
from pathlib import Path
import argparse, os, subprocess
from build import source_digest

ROOT = Path(__file__).resolve().parent

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--release', action='store_true')
    args = parser.parse_args()
    source = ROOT / 'sources/vectorcraft'
    destination = source / 'crates/engine/examples/pixa_vector_non_mac_acceptance.rs'
    fixture = (ROOT / 'vector-tests/native-workflow.rs').read_bytes()
    before = source_digest(source)
    # Refuse to overwrite another person's fixture; this file exists only during this check.
    with destination.open('xb') as output:
        output.write(fixture)
    try:
        environment = os.environ.copy()
        environment['CARGO_TARGET_DIR'] = str(ROOT / 'target')
        command = ['cargo', '+1.95.0', 'run', '--manifest-path', str(source / 'Cargo.toml'), '-p', 'vectorcraft-engine', '--features', 'bundled-fonts,heif', '--example', 'pixa_vector_non_mac_acceptance']
        if args.release:
            command.append('--release')
        result = subprocess.run(command, env=environment)
    finally:
        if destination.read_bytes() != fixture:
            raise RuntimeError('The temporary fixture was edited; preserving it for review')
        destination.unlink()
    if source_digest(source) != before:
        raise RuntimeError('Prepared source changed during the workflow audit; rebuild before packaging')
    raise SystemExit(result.returncode)

if __name__ == '__main__':
    main()
