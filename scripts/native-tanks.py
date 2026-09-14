"""Native Tanks keyboard/save lifecycle; run under Xvfb, separately from Wayland acceptance."""
from pathlib import Path
exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
with tempfile.TemporaryDirectory(prefix='arcade-tanks-') as tmp:
    state = Path(tmp) / 'state'
    save = state / 'omarchy-retro-arcade/tanks.json'
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data')
    def launch():
        app = subprocess.Popen([binary, '--game', 'tanks'], env=env)
        for _ in range(100):
            found = windows()
            if found and mapped(found[0]): break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found) == 1
        w = found[0]
        time.sleep(.5)
        x.XSetInputFocus(display,w,1,0); x.XFlush(display); time.sleep(.2)
        return app,w
    def read(): return json.loads(save.read_text())
    app,w = launch()
    try:
        key(ord('m'),True);key(ord('3'));key(ord('q'),True);app.wait(timeout=8)
        initial = read();assert initial['started'] and initial['mode']=='Local' and not initial['sound']
        app,w = launch()
        key(0x20);key(ord('q'),True);app.wait(timeout=8)
        assert read()['game'] == initial['game'], 'Paused Space changed the match'
        app,w = launch()
        key(0xff1b);key(0xff0d);key(0x20);time.sleep(.1);key(0xff1b)
        flight = read()['game'];assert flight['phase'] == 'Flying'
        key(0x20);key(ord('h'),True);key(0xff0d)
        key(ord('q'),True);app.wait(timeout=8)
        assert read()['game'] == flight, 'Shelf/reopen changed paused flight'
        app,w = launch();key(ord('q'),True);app.wait(timeout=8)
        assert read()['game'] == flight, 'Normal close/reopen changed flight'
        save.write_text('future-save-retain-exactly')
        app,w = launch();key(0x20);key(ord('q'),True);app.wait(timeout=8)
        assert save.read_text() == 'future-save-retain-exactly'
        print('PASS: Tanks keyboard handover/fire/pause, same-window shelf, exact mid-flight reopen, rejected-save preservation.')
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
