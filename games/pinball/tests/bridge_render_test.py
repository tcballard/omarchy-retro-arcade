"""Check real embedded artwork, full frame bounds, and graceful bridge shutdown."""
import os
from pathlib import Path
import select
import struct
import subprocess
import sys
import tempfile
import time

executable = str(Path(sys.argv[1]).resolve())
software = "--software" in sys.argv[2:]
with tempfile.TemporaryDirectory() as tmp:
    env = dict(os.environ, XDG_DATA_HOME=tmp, XDG_CONFIG_HOME=tmp,
               XDG_STATE_HOME=tmp, SDL_AUDIODRIVER="dummy")
    for key in tuple(env):
        if key.startswith("OMARCHY_TEST_"):
            del env[key]
    args = [executable, "--arcade-bridge", "--omarchy-table"]
    if software:
        args.append("-sw")
    with tempfile.TemporaryFile() as log:
        process = subprocess.Popen(args, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=log, env=env)
        deadline = time.monotonic() + 30

        def read_exact(size):
            data = bytearray()
            while len(data) < size:
                remaining = deadline - time.monotonic()
                assert remaining > 0, "Timed out waiting for Circuit frame"
                ready, _, _ = select.select([process.stdout], [], [], remaining)
                assert ready, "Circuit did not produce a complete frame"
                part = os.read(process.stdout.fileno(), size - len(data))
                assert part, "Circuit exited before completing a frame"
                data.extend(part)
            return data

        try:
            for _ in range(3):
                header = read_exact(12)
                assert header[:4] == b"OAR1", "Invalid frame magic"
                assert struct.unpack("<II", header[4:]) == (1152, 790)
                pixels = read_exact(1152 * 790 * 4)

            # The static upper playfield and right cabinet are absent in the
            # broken software render or clipped by an 800x556 drawable.
            for x0, y0, x1, y1 in [(170, 70, 650, 240), (820, 80, 1100, 280)]:
                visible = 0
                for y in range(y0, y1, 4):
                    for x in range(x0, x1, 4):
                        offset = (y * 1152 + x) * 4
                        visible += max(pixels[offset:offset + 3]) > 60
                assert visible > 500, "Circuit artwork is missing or clipped"

            process.communicate(b"quit\n", timeout=10)
            assert process.returncode == 0, "Bridge did not shut down cleanly"
            print("Circuit bridge: complete frames, " +
                  ("software artwork" if software else "visible full-size artwork") +
                  ", graceful quit")
        except BaseException:
            log.seek(0)
            print(log.read().decode(errors="replace")[-4000:], file=sys.stderr)
            raise
        finally:
            if process.poll() is None:
                process.kill()
            process.wait()
