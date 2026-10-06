#!/usr/bin/env python3
"""Genere ARBORESCENCE.md : un fichier par ligne, avec son role (premiere ligne « Responsabilite : » quand elle existe)."""
import re, subprocess, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
files = subprocess.run(['git', 'ls-files'], cwd=ROOT, capture_output=True, text=True, check=True).stdout.split()
SKIP = ('Cargo.lock', 'bun.lock', 'ui/public/vendor/', 'ui/public/fonts/')
PATTERN = re.compile(r'(?://!|//|#|/\*)\s*Responsabilite\s*:\s*(.+)')

def role(path: str) -> str:
    p = ROOT / path
    if p.suffix in ('.rs', '.ts', '.tsx', '.js', '.sh', '.py', '.css', '.html', '.toml', '.md'):
        try:
            for line in p.read_text(errors='ignore').splitlines()[:6]:
                m = PATTERN.search(line)
                if m:
                    return m.group(1).strip().rstrip('*/ ')
        except OSError:
            pass
    return ''

lines = ['# Arborescence', '', 'Genere par `tools/gen-arborescence.py` (ne pas editer a la main). Un fichier par ligne, avec son role.', '']
for f in sorted(files):
    if f.startswith(SKIP) or f in SKIP:
        continue
    r = role(f)
    lines.append(f'- `{f}`' + (f' — {r}' if r else ''))
(ROOT / 'ARBORESCENCE.md').write_text('\n'.join(lines) + '\n')
print(len(lines) - 4, 'fichiers')
