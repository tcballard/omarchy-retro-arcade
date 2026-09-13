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
        deadline = time.monotonic() + 60

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

            # Resize the same running worker: no second window or physics restart.
            # Invalid requests must not allocate or change the accepted surface.
            def command(text):
                process.stdin.write((text + "\n").encode())
                process.stdin.flush()

            def resized_frame(width, height):
                for _ in range(30):
                    header = read_exact(12)
                    assert header[:4] == b"OAR1"
                    w, h = struct.unpack("<II", header[4:])
                    assert 64 <= w <= 1600 and 64 <= h <= 1600 and w*h <= 1_000_000
                    frame = read_exact(w*h*4)
                    if (w,h) == (width,height):
                        return frame
                raise AssertionError("Resize did not produce requested dimensions")

            # Holding Space must animate the visible coil in its own housing,
            # keep it stable at full charge and restore it on release. Check
            # the actual pixels at the fixed bridge size, not the charge HUD.
            def coil(frame, w, h):
                portrait = w/h < 1.2
                scale = min(w/(1024 if portrait else 1536), h/(1230 if portrait else 1024))
                ox = (w-(1024 if portrait else 1536)*scale)/2
                oy = (h-(1230 if portrait else 1024)*scale)/2
                # Exclude the ball, its shadow and menu offset above the lower coil.
                return b"".join(frame[(y*w+int(ox+923*scale))*4:(y*w+int(ox+973*scale))*4]
                    for y in range(int(oy+905*scale),int(oy+970*scale)))

            def advance(seconds,w,h):
                until=time.monotonic()+seconds
                while True:
                    frame=resized_frame(w,h)
                    if time.monotonic()>=until: return frame

            for w,h in [(1152,790)]:
                command(f"resize {w} {h}")
                rest=coil(advance(.2,w,h),w,h)
                command("key 32 1 0")
                partial=coil(advance(.2,w,h),w,h)
                held=coil(advance(1.5,w,h),w,h)
                assert partial!=rest and partial!=held, "Spring does not track partial charge"
                assert held!=rest, "Spring does not visibly compress while Space is held"
                assert coil(advance(.3,w,h),w,h)==held, "Fully charged spring glitches while held"
                command("key 32 0 0")
                assert coil(advance(.2,w,h),w,h)==rest, "Spring does not return on release"
            print("Circuit spring: charge animation, full-charge hold and release at the fixed bridge size")

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
