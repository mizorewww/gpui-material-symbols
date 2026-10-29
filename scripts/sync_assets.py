#!/usr/bin/env python3
"""Download the complete, immutable upstream variable-font distribution."""
import argparse
import hashlib
import json
from pathlib import Path
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REVISION = '10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default=DEFAULT_REVISION)
    parser.add_argument('--verify', action='store_true')
    args = parser.parse_args()
    manifest_path = ROOT / 'assets/manifest.json'
    if args.verify:
        manifest = json.loads(manifest_path.read_text())
        for name, digest in manifest['sha256'].items():
            assert hashlib.sha256((ROOT / 'assets' / name).read_bytes()).hexdigest() == digest, name
        print('All upstream asset checksums verified')
        return
    if len(args.revision) != 40 or any(c not in '0123456789abcdef' for c in args.revision):
        parser.error('--revision must be a full lowercase Git commit SHA')
    base = f'https://raw.githubusercontent.com/google/material-design-icons/{args.revision}'
    files = {}
    for style in ('Outlined', 'Rounded', 'Sharp'):
        for ext in ('ttf', 'codepoints'):
            name = f'MaterialSymbols{style}.{ext}'
            url = f'{base}/variablefont/MaterialSymbols{style}%5BFILL%2CGRAD%2Copsz%2Cwght%5D.{ext}'
            print(f'Downloading {name}', flush=True)
            files[name] = urlopen(url, timeout=120).read()
    files['LICENSE'] = urlopen(f'{base}/LICENSE', timeout=30).read()
    # Validate all downloads before replacing any local files.
    for name, data in files.items():
        if name.endswith('.ttf'):
            assert data[:4] == b'\x00\x01\x00\x00', name
        elif name.endswith('.codepoints'):
            assert len(data.splitlines()) > 2500, name
    for name, data in files.items():
        (ROOT / 'assets' / name).write_bytes(data)
    manifest_path.write_text(json.dumps({'repository': 'https://github.com/google/material-design-icons', 'revision': args.revision, 'sha256': {n: hashlib.sha256(d).hexdigest() for n, d in files.items()}}, indent=2) + '\n')

if __name__ == '__main__':
    main()
