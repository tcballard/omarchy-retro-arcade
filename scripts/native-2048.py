"""Real X11 input, resume and credits checks; run under Xvfb. No app test hooks."""
from pathlib import Path
from PIL import ImageGrab

exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='arcade-2048-') as tmp:
    state = Path(tmp) / 'state'
    save = state / 'omarchy-retro-arcade/2048.json'; save.parent.mkdir(parents=True)
    original = {'version': 1, 'game': {'board': [[2, 2, 0, 0], [0]*4, [0]*4, [0]*4], 'score': 0, 'best': 0, 'continued': False, 'history': []}, 'reduced_motion': False}
    save.write_text(json.dumps(original))
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data')
    def read(): return json.loads(save.read_text())
    def launch():
        app = subprocess.Popen([binary, '--game', '2048'], env=env)
        for _ in range(100):
            found = windows()
            if found and mapped(found[0]): break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found) == 1
        w = found[0]; time.sleep(.5); x.XSetInputFocus(display,w,1,0); x.XFlush(display); time.sleep(.2)
        return app,w
    def capture(name):
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))
    app,w=launch()
    try:
        key(0xff51)
        # Opening without a window manager may first lose focus; explicitly resume.
        if read()['game']['score']==0: key(0xff1b);key(0xff51)
        assert read()['game']['score']==4, 'Native arrow input must merge the two starting tiles'
        capture('game')
        before=read()
        # A physical pointer drag must produce exactly one move.
        xt.XTestFakeMotionEvent(display,-1,600,500,0);x.XFlush(display);time.sleep(.1)
        xt.XTestFakeButtonEvent(display,1,1,0);x.XFlush(display);time.sleep(.1)
        xt.XTestFakeMotionEvent(display,-1,800,500,0);x.XFlush(display);time.sleep(.15)
        xt.XTestFakeButtonEvent(display,1,0,0);x.XFlush(display);time.sleep(.4)
        assert len(read()['game']['history'])==len(before['game']['history'])+1
        key(ord('z'),True);assert read()['game']['board']==before['game']['board']
        # Pause and help suppress moves, then resume with fresh input.
        key(0xff1b);paused=read();key(0xff54);assert read()==paused
        key(0xff1b);key(0xffbe);key(0xff54);assert read()==paused;capture('credits');key(0xff1b)
        saved=read();key(ord('h'),True);assert windows()==[w];capture('shelf')
        key(0xff0d);assert windows()==[w];assert read()==saved
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        app,w=launch();assert read()==saved
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        # A future/corrupt save must not be silently replaced by a new run or close.
        save.write_text('future-save-do-not-replace')
        app,w=launch();key(0xff51);capture('save-error');key(ord('q'),True);leave_anyway();app.wait(timeout=8)
        assert save.read_text()=='future-save-do-not-replace'
        print('PASS: 2048 native keys, drag, undo, pause/help isolation, same-window switch, reopen and corrupt-save retention.')
    finally:
        if app.poll() is None: app.kill();app.wait()
x.XCloseDisplay(display)
