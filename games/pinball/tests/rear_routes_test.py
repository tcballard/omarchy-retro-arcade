"""A moving ball can be trapped: require launch/target-pocket shots to reach play."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

executable=str(Path(sys.argv[1]).resolve())
shots=[('pillar pocket',(625,110,0,1,2,1)),
       ('rear slow',(650,80,-1,0,5,1)),
       ('rear medium',(650,80,-1,0,25,1)),
       ('rear fast',(650,80,-1,0,70,1)),
       ('target backs',(550,62,0,1,2,1)),
       ('bridge underpass',(370,65,0,1,12,1))]
for name,shot in shots:
    with tempfile.TemporaryDirectory() as tmp:
        env=dict(os.environ, XDG_CONFIG_HOME=tmp,XDG_DATA_HOME=tmp,
                 SDL_VIDEODRIVER='dummy',SDL_AUDIODRIVER='dummy',
                 OMARCHY_TEST_TICKS='2200',OMARCHY_TEST_SHOT='custom',
                 OMARCHY_TEST_VECTOR=' '.join(map(str,shot)))
        p=subprocess.run([executable,'--omarchy-table','-sw'],env=env,capture_output=True,text=True,timeout=100)
        assert p.returncode==0, name+' '+(p.stdout+p.stderr)[-2500:]
        assert 'REAR_EXIT 1' in p.stdout, name+' never reached the open playfield'
        print('PASS',name,flush=True)

# Exercise real feed/charge/contact timing too, without moving a ball onto the
# launcher or injecting a velocity. Start holds at different bounce phases.
for phase in (180,221,259):
    with tempfile.TemporaryDirectory() as tmp:
        env=dict(os.environ, XDG_CONFIG_HOME=tmp,XDG_DATA_HOME=tmp,
                 SDL_VIDEODRIVER='dummy',SDL_AUDIODRIVER='dummy',
                 OMARCHY_TEST_TICKS='2200',OMARCHY_TEST_SHOT='launch-feed',
                 OMARCHY_TEST_LAUNCH_AT=str(phase))
        p=subprocess.run([executable,'--omarchy-table','-sw'],env=env,capture_output=True,text=True,timeout=100)
        assert p.returncode==0, (p.stdout+p.stderr)[-2500:]
        assert 'REAR_EXIT 1' in p.stdout, f'Real launch at phase {phase} never reached play'
        print('PASS real feed/full launch',phase,flush=True)
