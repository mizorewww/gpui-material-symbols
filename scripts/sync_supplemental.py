#!/usr/bin/env python3
"""Extract all SVG variants absent from the fonts, from a pinned local Git clone."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
repo = sys.argv[1] if len(sys.argv) > 1 else str(root / '.research/material-design-icons')
manifest = json.loads((root / 'assets/manifest.json').read_text())
revision = manifest['revision']
def git(*args):
    return subprocess.check_output(['git', '-C', repo, *args])
font_names = {line.split()[0] for line in (root / 'assets/MaterialSymbolsOutlined.codepoints').read_text().splitlines()}
svg_names = set(git('ls-tree', '--name-only', f'{revision}:symbols/web').decode().splitlines())
missing = sorted(svg_names - font_names)
entries = []
for name in missing:
    for line in git('ls-tree', '-r', f'{revision}:symbols/web/{name}').decode().splitlines():
        metadata, path = line.split('\t')
        sha = metadata.split()[2]
        if path.endswith('.svg'):
            entries.append((f'{name}/{path}', sha))
pack = bytearray()
index = []
seen = {}
process = subprocess.Popen(['git', '-C', repo, 'cat-file', '--batch'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
for path, sha in sorted(entries):
    if sha not in seen:
        process.stdin.write((sha+'\n').encode()); process.stdin.flush()
        header = process.stdout.readline().split()
        size = int(header[2])
        data = process.stdout.read(size)
        assert process.stdout.read(1) == b'\n'
        assert b'<svg' in data
        seen[sha] = (len(pack), len(data))
        pack.extend(data)
    offset, size = seen[sha]
    index.append(f'{path}\t{offset}\t{size}\n')
process.stdin.close(); process.wait()
for name, data in {
    'supplemental.bin': bytes(pack),
    'supplemental.tsv': ''.join(index).encode(),
    'svg-names.txt': ('\n'.join(sorted(svg_names))+'\n').encode(),
}.items():
    (root / 'assets' / name).write_bytes(data)
    manifest['sha256'][name] = hashlib.sha256(data).hexdigest()
manifest['coverage'] = {'font_names': len(font_names), 'svg_names': len(svg_names), 'supplemental_names': len(missing), 'total_names': len(font_names | svg_names), 'supplemental_variants': len(entries)}
(root / 'assets/manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
print(manifest['coverage'], 'packed bytes:', len(pack))
