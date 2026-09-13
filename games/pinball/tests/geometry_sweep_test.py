"""Adversarial trajectories in the real engine; no alternative physics or clamping."""
import concurrent.futures
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())
shots = []
for speed in (25, 80):
    for angle in range(0, 360, 45):
        a = math.radians(angle)
        shots.append((540, 500, math.cos(a), math.sin(a), speed, 1))
for x, y in ((353, 42), (233, 181), (369, 344)):
    for dx, dy in ((1, 0), (-1, 0), (0, -1), (0, 1)):
        shots.append((x, y, dx, dy, 65, 2))
shots += [(372, 455, 0, -1, s, 1) for s in (35, 55, 90)]
shots += [(939, 841, dx, dy, speed, 1) for dx, dy, speed in
          ((0, 1, 20), (.15, -1, 35), (-.15, -1, 60), (0, -1, 100))]
shots += [(130, 900, 0, 1, 30, 1), (886, 880, 0, 1, 30, 1),
          (235, 700, 0, 1, 30, 1), (540, 985, 0, 1, 30, 1)]

def run(item):
    index, shot = item
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ, XDG_DATA_HOME=tmp, XDG_CONFIG_HOME=tmp,
                   SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
                   OMARCHY_TEST_TICKS="1600", OMARCHY_TEST_SHOT="custom",
                   OMARCHY_TEST_VECTOR=" ".join(map(str, shot)))
        result = subprocess.run([executable, "--omarchy-table", "-sw"],
                                env=env, capture_output=True, text=True, timeout=120)
        if result.returncode:
            return f"shot {index} {shot}: {result.stderr[-1800:]}"
        assert "GEOMETRY_AUDIT" in result.stdout
        assert "UPSTREAM_TABLE ticks=1600" in result.stdout
        return None

with concurrent.futures.ThreadPoolExecutor(max_workers=int(os.environ.get("OMARCHY_GEOMETRY_WORKERS", "1"))) as pool:
    failures = [failure for failure in pool.map(run, enumerate(shots)) if failure]
assert not failures, "\n".join(failures)
print(f"PASS {len(shots)} adversarial trajectories: both ramp layers, launches, lanes, drains; region and trapped-ball checks")
