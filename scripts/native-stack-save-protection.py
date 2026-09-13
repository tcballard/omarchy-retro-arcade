"""Verify rejected Stack saves through real input, shelf return and clean exit.

Run under xvfb-run with BINARY and an optional screenshot output directory.
All game data is disposable; existing player saves are never used.
"""
from pathlib import Path

from PIL import ImageGrab

# Reuse the repository's X11 input/window helpers, without its main test body.
helpers = (Path(__file__).parent / "native-check.py").read_text().split(
    "with tempfile.TemporaryDirectory"
)[0]
exec(compile(helpers, "native-input-helpers", "exec"))

out = Path(sys.argv[2]) if len(sys.argv) > 2 else None
if out:
    out.mkdir(parents=True, exist_ok=True)

try:
    for name, original in [
        ("invalid", b"broken JSON"),
        ("future", b'{"version":999,"future_run":"keep me"}'),
    ]:
        with tempfile.TemporaryDirectory(prefix="stack-save-protection-") as tmp:
            state = Path(tmp) / "state/omarchy-stack"
            state.mkdir(parents=True)
            save = state / "session.json"
            save.write_bytes(original)
            recovery = state / "session.rejected.json"
            if name == "future":
                recovery.mkdir()  # The old recovery copy fails at this destination.
            else:
                recovery.write_bytes(b"earlier recovery data")
            env = dict(
                os.environ,
                XDG_STATE_HOME=tmp + "/state",
                XDG_CONFIG_HOME=tmp + "/config",
                XDG_DATA_HOME=tmp + "/data",
                XDG_CACHE_HOME=tmp + "/cache",
            )
            # A Wayland desktop may launch this X11 harness under xvfb-run.
            # Keep the tested app on that virtual display, not the real desktop.
            env.pop("WAYLAND_DISPLAY", None)
            env.pop("WAYLAND_SOCKET", None)
            # The state theme takes precedence over the host's HOME fallback.
            theme = Path(tmp) / "state/omarchy/current/theme/colors.toml"
            theme.parent.mkdir(parents=True)
            theme.write_text(
                'background = "#f3f0e7"\nforeground = "#262b24"\naccent = "#526f3a"\n'
                if os.environ.get("STACK_LIGHT")
                else 'background = "#171c1a"\nforeground = "#e4e8df"\naccent = "#b3cb92"\n'
            )

            def unchanged():
                assert save.read_bytes() == original, "Original save was overwritten"
                if name == "future":
                    assert recovery.is_dir() and not list(recovery.iterdir())
                else:
                    assert recovery.read_bytes() == b"earlier recovery data"

            app = subprocess.Popen([binary, "--game", "stack", "--compact"], env=env)
            try:
                for _ in range(100):
                    found = windows()
                    if found:
                        break
                    assert app.poll() is None
                    time.sleep(.1)
                assert len(found) == 1
                window = found[0]
                x.XSetInputFocus(display, window, 1, 0)
                x.XFlush(display)
                time.sleep(.5)

                def capture(suffix):
                    if out:
                        attributes = WindowAttributes()
                        assert x.XGetWindowAttributes(display, window, C.byref(attributes))
                        bounds = (
                            attributes.x, attributes.y,
                            attributes.x + attributes.width, attributes.y + attributes.height,
                        )
                        ImageGrab.grab(bbox=bounds, xdisplay=os.environ["DISPLAY"]).save(
                            out / f"{name}-{suffix}.png"
                        )

                capture("menu")
                unchanged()
                key(0xff0d)  # Start an unsaved Marathon run.
                key(0x20)  # Drop a piece.
                time.sleep(2.3)  # Exercise the periodic save path.
                key(ord("p"))
                unchanged()
                capture("paused")
                key(ord("h"), True)
                leave_anyway()  # Explicitly leave the protected, unsaved session.
                assert windows() == [window]
                unchanged()
                key(0xff0d)  # Reopen Stack; protection must apply again.
                key(ord("s"))  # Start an unsaved Sprint run.
                key(0x20)
                key(ord("q"), True)
                leave_anyway()
                app.wait(timeout=10)
                assert app.returncode == 0
                unchanged()
            finally:
                if app.poll() is None:
                    app.kill()
                app.wait()
            print(f"PASS {name}: play, autosave, pause, shelf, reopen and exit preserve saves")
finally:
    x.XCloseDisplay(display)
