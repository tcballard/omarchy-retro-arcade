"""Left outlane loss route and production ball search, in the real engine with authored data."""
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())

def run(shot, ticks, vector=None):
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ, XDG_DATA_HOME=tmp, XDG_CONFIG_HOME=tmp,
                   SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
                   OMARCHY_TEST_TICKS=str(ticks), OMARCHY_TEST_SHOT=shot)
        if vector:
            env["OMARCHY_TEST_VECTOR"] = vector
        result = subprocess.run([executable, "--omarchy-table", "-sw"], env=env,
                                capture_output=True, text=True,
                                timeout=float(os.environ.get("OMARCHY_PHYSICS_TEST_TIMEOUT", "90")))
        assert result.returncode == 0, result.stdout[-3000:] + result.stderr[-3000:]
        return result.stdout

# A ball released just inside the outlane entrance is in a legal region and
# is lost at the drain; the static entry route is audited in every run.
out = run("custom", 900, "190 600 0 1 0 1")
assert "PLAYABLE_ROUTES 19 routes" in out, out[-2000:]
assert re.search(r"UPSTREAM_TABLE ticks=900 score=\d+ balls=2 ", out), out[-2000:]

# A ball at rest between two bumper skirts receives the ball-search kick and
# leaves; the usual five-second trap failure is waived only for this fixture.
out = run("ball-search", 1500)
searches = int(re.search(r"BALL_SEARCHES (\d+)", out).group(1))
assert searches >= 1, out[-2000:]
final = re.search(r"FINAL_BALL x=([\d.]+) y=([\d.]+)", out)
assert final is None or abs(float(final.group(1)) - 573.2) + abs(float(final.group(2)) - 236.8) > 60, out[-2000:]
print(f"PASS left outlane drains; ball search freed a rested ball after {searches} kick(s)")
