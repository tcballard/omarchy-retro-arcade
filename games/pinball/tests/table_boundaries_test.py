"""Launch real upstream balls at solid artwork boundaries, from the open playfield."""
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())
for shot, vector, axis, expected_sign in (
    ("rail_end_cap", "355 792 0 1 10 1", "dx", 1),
    ("module", "715 315 1 0 15 1", "dx", -1),
    ("guide", "325 500 -1 0 15 1", "dx", 1),
    ("sling_back_left", "257 680 1 0 15 1", "dx", -1),
    ("sling_back_right", "807 680 -1 0 15 1", "dx", 1),
):
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ, XDG_DATA_HOME=tmp, XDG_CONFIG_HOME=tmp,
                   SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
                   OMARCHY_TEST_TICKS="190", OMARCHY_TEST_SHOT="custom", OMARCHY_TEST_VECTOR=vector)
        result = subprocess.run([executable, "--omarchy-table", "-sw"],
                                env=env, capture_output=True, text=True, timeout=30)
        assert result.returncode == 0, result.stdout + result.stderr
        match = re.search(r"FINAL_BALL x=([-\d.]+) y=([-\d.]+) dx=([-\d.]+) dy=([-\d.]+)", result.stdout)
        assert match, result.stdout + result.stderr
        ball = dict(zip(("x", "y", "dx", "dy"), map(float, match.groups())))
        assert ball[axis] * expected_sign > 0.2, f"{shot}: ball passed through solid artwork: {ball}"
        if shot == "module":
            score = re.search(r"UPSTREAM_TABLE ticks=\d+ score=(\d+)", result.stdout)
            assert score and int(score[1]) >= 250, "Module face reflected without awarding the hit"
        print(shot, ball)
print("Solid table boundaries: round rail cap and both slingshot backs reflect upstream balls")
