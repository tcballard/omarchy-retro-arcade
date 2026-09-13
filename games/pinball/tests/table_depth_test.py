"""Compare real software-rendered frames above and below the raised ramp."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

executable = str(Path(sys.argv[1]).resolve())
for shot in ("under_ramp", "on_ramp"):
    frames = []
    for hidden in (False, True):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "frame.bmp"
            env = dict(os.environ, SDL_VIDEODRIVER="dummy", SDL_AUDIODRIVER="dummy",
                       XDG_CONFIG_HOME=tmp, XDG_DATA_HOME=tmp, OMARCHY_TEST_TICKS="181",
                       OMARCHY_TEST_SHOT="custom",
                       OMARCHY_TEST_VECTOR="350 40 0 1 0 "+("1" if shot=="under_ramp" else "2"), OMARCHY_TEST_SCREENSHOT=str(path))
            env.pop("OMARCHY_TEST_HIDE_BALL", None)
            if hidden:
                env["OMARCHY_TEST_HIDE_BALL"] = "1"
            result = subprocess.run([executable, "--omarchy-table", "-sw"], env=env,
                                    capture_output=True, text=True, timeout=30)
            assert result.returncode == 0, result.stdout + result.stderr
            frame = path.read_bytes()
            assert frame[:2] == b"BM", "Missing software-rendered screenshot"
            offset = int.from_bytes(frame[10:14], "little")
            pixels = frame[offset:]
            assert len(set(pixels)) > 64, "Table frame is blank"
            frames.append(pixels)
    changed = sum(a != b for a, b in zip(*frames))
    if shot == "under_ramp":
        assert changed == 0, f"Ground ball painted over raised ramp: {changed} changed bytes"
    else:
        assert changed > 100, "Ball riding on ramp was hidden by the deck"
    print(shot, "changed pixel bytes:", changed)
