"""Ground balls must use the lower mouth, never tube sides or the upper exit."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())
shots = [('left support', '252 85 0 1 15 1'),
         ('right support', '424 65 0 1 15 1'),
         ('reverse exit', '414 230 0 -1 15 1'),
         ('lower mouth', '372 455 0 -1 55 1')]
shots += [(f'launch {phase}', None) for phase in (180, 221, 259)]
for name, vector in shots:
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ,  XDG_CONFIG_HOME=tmp, XDG_DATA_HOME=tmp,
                   SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy',
                   OMARCHY_TEST_TICKS='2200', OMARCHY_TRACE_RAMP='1',
                   OMARCHY_TEST_SHOT='custom' if vector else 'launch-feed')
        if vector:
            env['OMARCHY_TEST_VECTOR'] = vector
        else:
            env['OMARCHY_TEST_LAUNCH_AT'] = name.split()[1]
        p = subprocess.run([executable, '--omarchy-table', '-sw'], env=env,
                           capture_output=True, text=True, timeout=100)
        output = p.stdout + p.stderr
        assert p.returncode == 0, name + '\n' + output[-3000:]
        entries = [line.split() for line in output.splitlines() if line.startswith('RAMP_ENTRY ')]
        for entry in entries:
            x, y = map(float, entry[2:4])
            assert 349 < x < 399 and 395 < y < 420, (name, entry)
        if name == 'lower mouth':
            assert entries, 'Legitimate lower entry blocked'
        if vector is None:
            assert 'REAR_EXIT 1' in output, name + ' did not reach play'
        print('PASS', name, 'legal entries', len(entries), flush=True)
