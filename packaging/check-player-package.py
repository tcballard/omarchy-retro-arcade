#!/usr/bin/env python3
"""Validate an extracted Arch player package and print its installed footprint."""
import argparse
from pathlib import Path


def check(root, manifest):
    entries = [line.split('\t') for line in manifest.read_text().splitlines()
               if line and not line.startswith('#')]
    expected = {'usr/' + destination for _, destination in entries}
    assert len(expected) == len(entries), 'Duplicate installed paths'
    expected.update({'usr/libexec/omarchy-retro-arcade/stockfish',
                     'usr/share/licenses/omarchy-retro-arcade/Stockfish.txt'})
    metadata = {'.PKGINFO', '.BUILDINFO', '.MTREE'}
    actual = set()
    sizes = {}
    for path in root.rglob('*'):
        assert not path.is_symlink(), f'Unexpected symlink: {path}'
        if path.is_file():
            name = path.relative_to(root).as_posix()
            if name not in metadata:
                actual.add(name)
                sizes[name] = path.stat().st_size
    assert actual == expected, (f'Missing: {sorted(expected - actual)}; '
                                f'Unexpected: {sorted(actual - expected)}')
    for name in expected:
        assert sizes[name] > 0, f'Empty payload file: {name}'
        if name.startswith(('usr/bin/', 'usr/libexec/')):
            assert (root / name).stat().st_mode & 0o111, f'Not executable: {name}'
    docs = sum(size for name, size in sizes.items() if name.startswith('usr/share/doc/'))
    assert docs < 1024 * 1024, f'Player documentation exceeds 1 MiB: {docs}'
    print(f'Player payload: {sum(sizes.values()) / 1048576:.2f} MiB; '
          f'documentation: {docs / 1024:.1f} KiB')
    for name, size in sorted(sizes.items(), key=lambda item: item[1], reverse=True)[:5]:
        print(f'{size / 1048576:7.2f} MiB  {name}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    args = parser.parse_args()
    check(args.root, Path(__file__).with_name('player-files.tsv'))
