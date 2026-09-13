"""Actual upstream input impulses, tilt penalties, and next-ball recovery."""
import os, subprocess, sys, tempfile
with tempfile.TemporaryDirectory() as tmp:
    env=dict(os.environ,XDG_CONFIG_HOME=tmp,XDG_DATA_HOME=tmp,SDL_VIDEODRIVER='dummy',SDL_AUDIODRIVER='dummy',OMARCHY_TEST_SHOT='nudge',OMARCHY_TEST_TICKS='630')
    p=subprocess.run([sys.argv[1],'--omarchy-table','-sw'],env=env,capture_output=True,text=True,timeout=60)
    assert p.returncode==0,p.stdout+p.stderr
    for result in ['impulses/release/pause/focus','warning','no premature tilt at 1.5 seconds','tilt disables flippers/scoring','drain restores next ball']:
        assert f'NUDGE {result} PASS' in p.stdout,p.stdout
    print(p.stdout)

    env['OMARCHY_NUDGE_RELEASE_EARLY']='1'
    p=subprocess.run([sys.argv[1],'--omarchy-table','-sw'],env=env,capture_output=True,text=True,timeout=60)
    assert p.returncode==0,p.stdout+p.stderr
    assert 'NUDGE early release prevents delayed tilt PASS' in p.stdout,p.stdout
    print(p.stdout)
