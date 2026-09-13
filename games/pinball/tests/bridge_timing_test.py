"""A slow consumer preserves elapsed steps up to the deliberate 100ms stall cap."""
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile
import time

with tempfile.TemporaryDirectory() as tmp, tempfile.TemporaryFile() as log:
    env=dict(os.environ,XDG_CONFIG_HOME=tmp,XDG_DATA_HOME=tmp,
             SDL_AUDIODRIVER="dummy",OMARCHY_CHECK_TIMING="1")
    p=subprocess.Popen([str(Path(sys.argv[1]).resolve()),"--arcade-bridge","--omarchy-table"],
        stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=log,env=env)
    def frame():
        header=p.stdout.read(12)
        assert header[:4]==b"OAR1"
        w,h=struct.unpack("<II",header[4:])
        assert len(p.stdout.read(w*h*4))==w*h*4
    try:
        for _ in range(10): frame()
        for _ in range(40):
            time.sleep(.04)
            frame()
        p.communicate(b"quit\n",timeout=10)
        assert p.returncode==0
        log.seek(0)
        samples=[tuple(map(float,m)) for m in re.findall(rb"CIRCUIT_CLOCK ([\d.]+) ([\d.]+) ([\d.]+)",log.read())]
        assert len(samples)>20
        wall=samples[-1][0]-samples[10][0]
        simulation=samples[-1][1]-samples[10][1]
        assert wall>1000
        # Very slow software renderers may exceed the intentional stall cap.
        # Check the engine consumed every bounded elapsed step, not an uncapped
        # wall-time promise. The old loop truncates steps to about16.7ms.
        steps=[sample[2] for sample in samples[11:]]
        assert max(steps)>30, 'Slow consumer did not exercise retained elapsed time'
        expected=sum(min(step,100) for step in steps)
        assert abs(simulation-expected)<max(2,expected*.005), (wall,simulation,expected)
        print(f"Slow consumer: {wall:.0f} ms wall, {simulation:.0f} ms simulation")
    finally:
        if p.poll() is None: p.kill()
        p.wait()
