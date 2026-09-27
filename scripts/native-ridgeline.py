"""Native Ridgeline keyboard play, lifecycle and save protection. Run under Xvfb.
argv: binary, output directory, optional variant (dark, light, compact, 200).
RIDGELINE_FIXTURE names the fixture example binary (default target/release/examples/fixture).
No engine test hooks: the save is written by the production fixture example."""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
variant = sys.argv[3] if len(sys.argv) > 3 else 'dark'
fixture = os.environ.get('RIDGELINE_FIXTURE', str(Path(__file__).parent.parent / 'target/release/examples/fixture'))
with tempfile.TemporaryDirectory(prefix='arcade-ridgeline-') as tmp:
    state = Path(tmp) / 'state'
    save = state / 'omarchy-retro-arcade/ridgeline.json'
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp + '/config', XDG_DATA_HOME=tmp + '/data',
               WINIT_X11_SCALE_FACTOR='2' if variant == '200' else '1')
    extra = ['--compact'] if variant in ('compact', '200') else []
    if variant == 'light':
        theme = state / 'omarchy/current/theme/colors.toml'; theme.parent.mkdir(parents=True)
        theme.write_text('background = "#f3f0e7"\nforeground = "#262b24"\naccent = "#526f3a"\n')
    def read(): return json.loads(save.read_text())
    def battle(): return read()['battle']
    def launch():
        # Keep the pointer off the board so only keyboard input moves the cursor.
        xt.XTestFakeMotionEvent(display, -1, 2, 2, 0); x.XFlush(display)
        app = subprocess.Popen([binary, '--game', 'ridgeline'] + extra, env=env)
        for _ in range(120):
            found = windows()
            if found and mapped(found[0]): break
            assert app.poll() is None, 'app exited during launch'
            time.sleep(.1)
        assert len(found) == 1, found
        w = found[0]; time.sleep(.8); x.XSetInputFocus(display, w, 1, 0); x.XFlush(display); time.sleep(.3)
        return app, w
    def capture(name):
        attr = WindowAttributes(); x.XGetWindowAttributes(display, w, C.byref(attr))
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x, attr.y, attr.x + attr.width, attr.y + attr.height)).save(out / (name + '.png'))
    def ready(title):
        for _ in range(80):
            name = C.c_void_p(); x.XFetchName(display, w, C.byref(name))
            value = C.string_at(name).decode(errors='replace') if name.value else ''
            if name.value: x.XFree(name)
            if value == title: time.sleep(.4); return
            time.sleep(.1)
        raise AssertionError(('title', title, value))
    def quit(app):
        key(ord('q'), True); app.wait(timeout=10); assert app.returncode == 0, ('exit status', app.returncode)
    def mouse(px, py, button=1):
        xt.XTestFakeMotionEvent(display, -1, px, py, 0); x.XFlush(display); time.sleep(.1)
        xt.XTestFakeButtonEvent(display, button, 1, 0); x.XFlush(display); time.sleep(.06)
        xt.XTestFakeButtonEvent(display, button, 0, 0); x.XFlush(display); time.sleep(.25)
    app, w = launch()
    try:
        # A fresh campaign opens on the map list; closing writes an empty campaign.
        capture('maps'); key(0xff1b); key(0x20)
        quit(app)
        assert read()['battle'] is None, 'fresh campaign saved a battle'
        # Production fixture: First Terrace, waiting for wave 3 after the replay's builds.
        subprocess.run([fixture, '1', '3', '0', str(save)], check=True)
        before = battle(); assert before['phase'] == 'Waiting' and before['wave'] == 2
        occupied = {(t['x'], t['y']) for t in before['towers']}
        target = next(c for c in [(10, 6), (10, 5), (11, 6), (8, 5), (8, 6), (11, 5), (3, 5), (2, 6), (16, 6), (16, 7)] if c not in occupied)
        if variant == 'dark':
            # Exercise the real pointer path at the standard 1280x900 layout.
            # The engine-level egui test also checks mouse placement at other sizes.
            app, w = launch()
            mouse(1100, 254)  # Cannon card in the right rail.
            mouse(round(10 + (target[0] + .5) * 48.4), round(175 + (target[1] + .5) * 48.4))
            placed = battle()
            assert len(placed['towers']) == len(before['towers']) + 1, 'mouse did not build'
            assert (placed['towers'][-1]['x'], placed['towers'][-1]['y']) == target
            assert placed['credits'] == before['credits'] - 50
            capture('mouse-built')
            mouse(round(10 + (target[0] + .5) * 48.4), round(175 + (target[1] + .5) * 48.4))
            # The fixture has 66 credits: sell the new Cannon before upgrading
            # an existing tier-one Cannon. The inspector is below the shop.
            mouse(1210, 710)
            sold = battle()
            assert len(sold['towers']) == len(before['towers']), 'mouse did not sell'
            assert sold['credits'] == before['credits'] - 50 + 35
            capture('mouse-sold')
            candidate = next(t for t in before['towers'] if t['tier'] == 0)
            mouse(round(10 + (candidate['x'] + .5) * 48.4), round(175 + (candidate['y'] + .5) * 48.4))
            mouse(1075, 710)
            upgraded = battle()
            assert next(t for t in upgraded['towers'] if t['id'] == candidate['id'])['tier'] == 1, 'mouse did not upgrade'
            assert upgraded['credits'] == sold['credits'] - 45
            capture('mouse-upgraded')
            quit(app)
            subprocess.run([fixture, '1', '3', '0', str(save)], check=True)
        app, w = launch(); capture('waiting')
        assert before['credits'] >= 50, before['credits']
        # Keyboard placement: 1 selects Cannon, arrows move the shared cursor, Enter builds.
        key(ord('1'))
        dx, dy = target[0] - 10, target[1] - 6
        for _ in range(abs(dx)): key(0xff53 if dx > 0 else 0xff51)
        for _ in range(abs(dy)): key(0xff54 if dy > 0 else 0xff52)
        capture('placing'); key(0xff0d)
        key(ord('h'), True); ready('Omarchy Arcade')
        built = battle()
        assert len(built['towers']) == len(before['towers']) + 1, 'Enter did not build'
        assert (built['towers'][-1]['x'], built['towers'][-1]['y']) == target, (built['towers'][-1], target)
        assert built['credits'] == before['credits'] - 50, 'build cost'
        key(0xff0d); ready('Ridgeline - Omarchy Arcade')
        # Escape cancels a pending placement before any pause; nothing is spent.
        key(ord('2')); key(0xff1b)
        key(0x20); time.sleep(1.2); capture('running')
        key(0xff1b)
        paused = battle(); assert paused['phase'] == 'Running' and paused['tick'] > 0, paused['tick']
        assert paused['credits'] >= built['credits'] and len(paused['towers']) == len(built['towers']), 'cancelled placement spent credits'
        capture('paused')
        # Paused time, Space and Enter never advance combat.
        time.sleep(.8); key(0x20); key(ord('u'))
        key(ord('h'), True); ready('Omarchy Arcade')
        assert battle()['tick'] == paused['tick'], 'paused combat advanced'
        key(0xff0d); ready('Ridgeline - Omarchy Arcade'); capture('reopened-paused')
        quit(app)
        exact = read(); assert exact['battle']['tick'] == paused['tick'], 'close changed the paused tick'
        # A new process restores the same in-flight state, paused.
        app, w = launch(); time.sleep(.6); quit(app)
        assert read() == exact, 'reopen changed the saved battle'
        # Resume by keyboard and let the wave progress; closing saves the exact state.
        # (A second Escape could meet a legitimate >1 s cold-start stall pause and resume.)
        app, w = launch(); key(0xff1b); time.sleep(2.)
        quit(app)
        assert battle()['tick'] > exact['battle']['tick'], 'resumed wave did not advance'
        # Unsupported data is preserved exactly; nothing overwrites it.
        save.write_text('future-save-do-not-replace')
        app, w = launch(); capture('recovery'); key(0x20); key(0xff1b); key(ord('1')); key(0xff0d)
        quit(app)
        assert save.read_text() == 'future-save-do-not-replace', 'protected save was overwritten'
        print(f'PASS ({variant}): Ridgeline keyboard build, cancel, start/pause, paused isolation, same-window shelf return, exact reopen, protected save.')
    finally:
        if app.poll() is None: app.kill(); app.wait()
x.XCloseDisplay(display)
