#!/usr/bin/env python3
"""Download the complete, immutable upstream variable-font distribution."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / 'assets'
DEFAULT_REVISION = '10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7'
STYLES = ('Outlined', 'Rounded', 'Sharp')
FONT_FILES = [f'MaterialSymbols{s}.{e}' for s in STYLES for e in ('ttf', 'codepoints')] + ['LICENSE']
SUPPLEMENTAL_FILES = ['supplemental.bin', 'supplemental.tsv', 'svg-names.txt']


def fail(message):
    sys.exit(f'error: {message}')


def write_atomic(path, data):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_bytes(data)
    os.replace(temporary, path)


def names(codepoints):
    return {line.split()[0] for line in codepoints.read_text().splitlines()}


def verify(manifest):
    expected = set(FONT_FILES + SUPPLEMENTAL_FILES)
    if set(manifest.get('sha256', {})) != expected:
        fail(f'manifest must hash exactly {sorted(expected)}; rerun sync_supplemental.py')
    if manifest.get('supplemental_revision') != manifest.get('revision'):
        fail('supplemental SVGs were extracted from a different revision than the fonts')
    for name, digest in manifest['sha256'].items():
        if hashlib.sha256((ASSETS / name).read_bytes()).hexdigest() != digest:
            fail(f'{name} does not match its recorded SHA-256')

    font_names = names(ASSETS / 'MaterialSymbolsOutlined.codepoints')
    for style in STYLES[1:]:
        if names(ASSETS / f'MaterialSymbols{style}.codepoints') != font_names:
            fail(f'{style} codepoints name a different symbol set than Outlined')
    svg_names = set((ASSETS / 'svg-names.txt').read_text().split())
    pack_len = (ASSETS / 'supplemental.bin').stat().st_size
    pack = (ASSETS / 'supplemental.bin').read_bytes()
    previous, variants, supplemental_names = '', 0, set()
    for number, line in enumerate((ASSETS / 'supplemental.tsv').read_text().splitlines(), 1):
        path, offset, length = line.split('\t')
        offset, length = int(offset), int(length)
        if not previous < path:
            fail(f'supplemental.tsv:{number} is not strictly sorted')
        if offset + length > pack_len or not pack[offset:offset + length].startswith(b'<svg'):
            fail(f'supplemental.tsv:{number} does not address an SVG in supplemental.bin')
        previous, variants = path, variants + 1
        supplemental_names.add(path.split('/', 1)[0])
    if supplemental_names != svg_names - font_names:
        fail('supplemental.tsv does not cover exactly the SVG-only names')
    coverage = {
        'font_names': len(font_names),
        'svg_names': len(svg_names),
        'supplemental_names': len(supplemental_names),
        'total_names': len(font_names | svg_names),
        'supplemental_variants': variants,
    }
    if manifest.get('coverage') != coverage:
        fail(f'recorded coverage {manifest.get("coverage")} differs from assets {coverage}')
    print(f'All upstream asset checksums and coverage verified at {manifest["revision"]}')


def verify_or_fail(manifest):
    try:
        verify(manifest)
    except (OSError, ValueError, KeyError) as error:
        fail(f'cannot verify assets: {error!r}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default=DEFAULT_REVISION)
    parser.add_argument('--verify', action='store_true', help='check local assets only; no network')
    args = parser.parse_args()
    manifest_path = ASSETS / 'manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    if args.verify:
        verify_or_fail(manifest)
        return
    if len(args.revision) != 40 or any(c not in '0123456789abcdef' for c in args.revision):
        parser.error('--revision must be a full lowercase Git commit SHA')
    base = f'https://raw.githubusercontent.com/google/material-design-icons/{args.revision}'
    files = {}
    for style in STYLES:
        for ext in ('ttf', 'codepoints'):
            name = f'MaterialSymbols{style}.{ext}'
            url = f'{base}/variablefont/MaterialSymbols{style}%5BFILL%2CGRAD%2Copsz%2Cwght%5D.{ext}'
            print(f'Downloading {name}', flush=True)
            files[name] = urlopen(url, timeout=120).read()
    files['LICENSE'] = urlopen(f'{base}/LICENSE', timeout=30).read()
    # Validate all downloads before replacing any local files.
    for name, data in files.items():
        if name.endswith('.ttf') and data[:4] != b'\x00\x01\x00\x00':
            fail(f'{name} is not a TrueType font')
        if name.endswith('.codepoints') and len(data.splitlines()) <= 2500:
            fail(f'{name} is implausibly short')
    for name, data in files.items():
        write_atomic(ASSETS / name, data)
    sha256 = manifest.get('sha256', {})
    sha256.update({n: hashlib.sha256(d).hexdigest() for n, d in files.items()})
    manifest.update(repository='https://github.com/google/material-design-icons', revision=args.revision, sha256=sha256)
    write_atomic(manifest_path, (json.dumps(manifest, indent=2) + '\n').encode())
    if manifest.get('supplemental_revision') != args.revision:
        print('Fonts updated; now run sync_supplemental.py for the same revision.')


if __name__ == '__main__':
    main()
