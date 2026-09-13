"""Real feed rests on the visible coil head; passive targets do not add energy."""
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

out, contacts = run('natural resting contact', 900, OMARCHY_TEST_SHOT='launch-feed',
                    OMARCHY_TEST_LAUNCH_AT='99999')
rest = [c for c in contacts if c[2] == 'plunger']
assert rest, 'Ball did not reach the spring head'
assert abs(float(rest[-1][6])+11.25-873) < .1, 'Ball bottom misses visible spring head'
assert abs(float(rest[-1][5])-939) < 2, 'Ball is not centred over the spring'
assert float(rest[-1][4]) < .5, 'Ball does not settle on the launcher'
for name in ('plunger', 'plunger-repeat'):
    out, _ = run(name, 1000, OMARCHY_TEST_SHOT=name)
    marker = 'PLUNGER_FULL_LAUNCH 1' if name == 'plunger' else 'PLUNGER_RECHARGE 1'
    assert marker in out, name + ' failed contact/charge recovery'
