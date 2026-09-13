"""Exercise real upstream collisions, ramp layers, drains and rendering without a DAT."""
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())

def run(shot=None):
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ,  XDG_DATA_HOME=tmp, XDG_CONFIG_HOME=tmp,
                   SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
                   OMARCHY_TEST_TICKS="1000" if shot else "21600")
        env.pop("OMARCHY_TEST_SHOT", None)
        if shot:
            env["OMARCHY_TEST_SHOT"] = shot
        result = subprocess.run([executable, "--omarchy-table", "-sw"],
                                env=env, capture_output=True, text=True,
                                timeout=float(os.environ.get("OMARCHY_PHYSICS_TEST_TIMEOUT", "90")))
        assert result.returncode == 0, result.stdout[-4000:] + result.stderr[-4000:]
        match = re.search(r"UPSTREAM_TABLE ticks=(\d+) score=(\d+) balls=(\d+) ramps=(\d+) orbits=(\d+) targets=(\d+)", result.stdout)
        assert match, result.stdout[-4000:]
        if shot == "ramp-complete":
            assert "RAMP_COMPLETE 1" in result.stdout, "Strong shot did not traverse the tube and return to ground at its exit"
        if shot == "plunger":
            assert "PLUNGER_FULL_LAUNCH 1" in result.stdout, "Full charge expired before ball contact"
        if shot == "plunger-repeat":
            assert "PLUNGER_RECHARGE 1" in result.stdout, "Previous release timer interrupted a new charge"
        assert not list(Path(tmp).rglob("*.DAT"))
        values = tuple(map(int, match.groups()))
        print(shot or "full game", values)
        return values

_, score, balls, _, _, _ = run()
assert score >= 100, "No scoring collisions registered"
assert balls == 0, "Three-ball game did not finish"
_, score, _, ramps, _, _ = run("ramp")
assert ramps >= 1 and score >= 1500, "Raised ramp shot did not register"
_, score, _, _, _, targets = run("target")
assert targets != 0 and score >= 250, "Target collision did not register"
_, score, _, _, orbits, _ = run("orbit")
assert orbits >= 1 and score >= 1000, "Orbit sensor did not register"
run("ramp-complete")
# This position is now on the elevated chute; the closed ground backboard
# makes the old mask-1 approach impossible. Exercise its actual layer.
run("ramp-exit-side")
for wall in ("raised-top", "raised-left", "raised-right"):
    run(wall)
_, _, balls, _, _, _ = run("drain")
assert balls == 2, "Drain did not feed the next ball"
print("Upstream physics: full game, ramp, target, orbit, drain; bounded finite ball state; no external DAT")
