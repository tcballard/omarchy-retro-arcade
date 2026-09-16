"""Native Minesweeper lifecycle under Xvfb; separate from Wayland acceptance."""
from pathlib import Path
exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
with tempfile.TemporaryDirectory(prefix='arcade-mines-') as tmp:
    state = Path(tmp) / 'state'
    save = state / 'omarchy-retro-arcade/minesweeper.json'
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data')
    def launch():
        app = subprocess.Popen([binary, '--game', 'minesweeper', '--compact'], env=env)
        for _ in range(100):
            found = windows()
            if found and mapped(found[0]): break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found) == 1
        w = found[0]
        x.XSetInputFocus(display,w,1,0); x.XFlush(display); time.sleep(.5)
        return app,w
    def read(): return json.loads(save.read_text())
    app,w = launch()
    try:
        key(ord('f')); assert read()['game']['cells'][0]['flagged']
        key(0x20); assert read()['game']['status'] == 'Ready'
        key(ord('f')); key(0x20)
        assert read()['game']['status'] == 'Playing'
        key(0xff1b); key(ord('h'),True)
        paused = read()
        key(0xff0d); assert windows() == [w]
        key(0x20); key(ord('f')); key(ord('q'),True); app.wait(timeout=8)
        assert read()['game'] == paused['game'], 'Paused input changed board or timer'
        app,w = launch(); key(ord('q'),True); app.wait(timeout=8)
        assert read()['game'] == paused['game'], 'Reopening changed paused board'
        save.write_text('future-save-retain-exactly')
        app,w = launch(); key(0x20); key(ord('q'),True); app.wait(timeout=8)
        assert save.read_text() == 'future-save-retain-exactly'
        print('PASS: Minesweeper keyboard, pause, same-window shelf, exact resume and rejected-save preservation.')
    finally:
        if app.poll() is None: app.kill(); app.wait()
x.XCloseDisplay(display)
