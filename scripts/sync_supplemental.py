#!/usr/bin/env python3
"""Extract all SVG variants absent from the fonts, from a pinned local Git clone."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
repo = sys.argv[1] if len(sys.argv) > 1 else str(root / '.research/material-design-icons')
manifest = json.loads((root / 'assets/manifest.json').read_text())
revision = manifest['revision']
def fail(message):
    if 'process' in globals():
        process.kill()
    sys.exit(f'error: {message}')
def write_atomic(path, data):
    temporary = path.with_name(path.name + '.tmp')
    temporary.write_bytes(data)
    os.replace(temporary, path)
def git(*args):
    return subprocess.check_output(['git', '-C', repo, *args])
if subprocess.run(['git', '-C', repo, 'cat-file', '-e', f'{revision}^{{commit}}']).returncode:
    fail(f'{repo} lacks commit {revision}; fetch it first (see docs/maintenance.md)')
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
        if len(header) != 3 or header[1] != b'blob':
            fail(f'cannot read blob {sha} for {path}: {header}')
        size = int(header[2])
        data = process.stdout.read(size)
        if process.stdout.read(1) != b'\n' or not data.startswith(b'<svg'):
            fail(f'{path} is not a complete SVG blob')
        seen[sha] = (len(pack), len(data))
        pack.extend(data)
    offset, size = seen[sha]
    index.append(f'{path}\t{offset}\t{size}\n')
process.stdin.close()
if process.wait():
    fail('git cat-file --batch failed')
for name, data in {
    'supplemental.bin': bytes(pack),
    'supplemental.tsv': ''.join(index).encode(),
    'svg-names.txt': ('\n'.join(sorted(svg_names))+'\n').encode(),
}.items():
    write_atomic(root / 'assets' / name, data)
    manifest['sha256'][name] = hashlib.sha256(data).hexdigest()
manifest['supplemental_revision'] = revision
manifest['coverage'] = {'font_names': len(font_names), 'svg_names': len(svg_names), 'supplemental_names': len(missing), 'total_names': len(font_names | svg_names), 'supplemental_variants': len(entries)}
write_atomic(root / 'assets/manifest.json', (json.dumps(manifest, indent=2)+'\n').encode())
print(manifest['coverage'], 'packed bytes:', len(pack))
