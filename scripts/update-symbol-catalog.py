#!/usr/bin/env python3
"""Refresh example metadata against this repository's pinned, complete icon names."""
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
SYMBOLS = ROOT / 'assets'
URL = 'https://fonts.google.com/metadata/icons?key=material_symbols&incomplete=true'
raw = urllib.request.urlopen(URL, timeout=60).read().decode()
metadata = json.loads(raw[raw.index('{'):])
icons = {i['name']: i for i in metadata['icons']
         if 'Material Symbols Outlined' not in i.get('unsupported_families', [])}
names = set((SYMBOLS / 'svg-names.txt').read_text().split())
for path in SYMBOLS.glob('*.codepoints'):
    names.update(line.split()[0] for line in path.read_text().splitlines())
rows = []
for name in sorted(names):
    icon = icons.get(name, {})
    category = icon.get('categories', ['Other'])[0].replace('Audio&Video', 'Audio & Video')
    tags = ' '.join(' '.join(icon.get('tags', [])).split()) or name.replace('_', ' ')
    rows.append(f"{name}\t{category}\t{icon.get('popularity', 0)}\t{tags}\n")
target = ROOT / 'examples/material_symbols/assets/catalog.tsv'
target.write_text(''.join(rows))
print(f'Wrote {len(rows)} entries to {target}')
