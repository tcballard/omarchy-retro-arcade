"""Passive scoring targets rebound without adding powered bumper energy."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())

def run(name, ticks, **extra):
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ, XDG_CONFIG_HOME=tmp, XDG_DATA_HOME=tmp,
                   SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy',
                   OMARCHY_TEST_TICKS=str(ticks), OMARCHY_TRACE_CONTACTS='1',
                   OMARCHY_TRACE='1', **extra)
        result = subprocess.run([executable, '--omarchy-table', '-sw'], env=env,
                                capture_output=True, text=True, timeout=90)
        assert result.returncode == 0, name + (result.stdout+result.stderr)[-2500:]
        contacts = [line.split() for line in result.stderr.splitlines() if line.startswith('CONTACT ')]
        print('PASS', name, flush=True)
        return result.stdout, contacts

shots = [(f'upper target {i}', f'{512+i*28} 163 0 -1 15 1', f'target{i}') for i in range(4)]
shots += [(f'side module {i}', f'{715-i*6} {315+i*25} 1 0 15 1', f'module{i}') for i in range(4)]
shots += [('powered bumper', '470 330 0 -1 20 1', 'bumper0')]
for name, vector, component in shots:
    out, contacts = run(name, 550, OMARCHY_TEST_SHOT='custom', OMARCHY_TEST_VECTOR=vector)
    hits = [c for c in contacts if c[2] == component]
    assert hits, f'{name}: fixture missed collision'
    if component.startswith('bumper'):
        assert any(float(c[4]) > float(c[3])+1 for c in hits), 'Powered bumper lost its kick'
    else:
        assert all(float(c[4]) <= float(c[3])+.01 for c in hits), f'{name}: passive hit injected energy'
        assert 'score=0 ' not in out, f'{name}: passive target no longer scores'
print('PASS passive scoring faces, powered bumper response')
