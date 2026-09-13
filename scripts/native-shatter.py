"""Native Shatter lifecycle and input. Run under Xvfb; no engine test hooks."""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='arcade-shatter-') as tmp:
    state = Path(tmp) / 'state'
    save = state / 'omarchy-retro-arcade/shatter.json'
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data')
    def read(): return json.loads(save.read_text())
    def launch():
        app = subprocess.Popen([binary, '--game', 'shatter'], env=env)
        for _ in range(100):
            found = windows()
            if found and mapped(found[0]): break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found) == 1
        w=found[0];time.sleep(.5);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.2)
        return app,w
    def capture(name):
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))
    app,w=launch()
    try:
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        original=read()
        # An existing campaign always opens paused, regardless of WM focus order.
        app,w=launch();key(0x20);key(ord('m'),True)
        assert read()['campaign']==original['campaign'], 'Paused Space must not serve'
        key(0xff1b);capture('ready-to-serve');key(0x20);time.sleep(.15);capture('playing');key(0xff1b)
        playing=read()['campaign'];assert playing['phase']=='Playing';assert len(playing['balls'])==1
        capture('paused-flight')
        key(0xff53,hold=.1);key(0x20);key(ord('m'),True)
        assert read()['campaign']==playing, 'Paused gameplay must not leak'
        # Resume and move entirely by pointer, then pause using the toolbar.
        click(105,65)
        xt.XTestFakeMotionEvent(display,-1,900,600,0);x.XFlush(display);time.sleep(.3)
        click(105,65)
        moved=read()['campaign'];assert moved['paddle']>playing['paddle']+40, (playing['paddle'],moved['paddle'])
        playing=moved
        key(ord('h'),True);assert windows()==[w];capture('shelf')
        key(0xff0d);assert windows()==[w];assert read()['campaign']==playing
        key(ord('q'),True);app.wait(timeout=8)
        app,w=launch();assert read()['campaign']==playing;capture('continued')
        key(ord('q'),True);app.wait(timeout=8)
        save.write_text('future-save-do-not-replace')
        app,w=launch();key(0x20);key(ord('q'),True);app.wait(timeout=8)
        assert save.read_text()=='future-save-do-not-replace'
        print('PASS: Shatter launch/pause, held-input isolation, same-window shelf, exact reopen, corrupt-save protection.')
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
