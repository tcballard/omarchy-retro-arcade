"""Native mouse regression checks; run under Xvfb, not an Omarchy desktop substitute."""
from pathlib import Path
exec(compile((Path(__file__).parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
from PIL import ImageGrab
out = Path(sys.argv[2] if len(sys.argv) > 2 else '/tmp/arcade-mouse')
out.mkdir(parents=True, exist_ok=True)
compact = os.environ.get('ARCADE_TEST_COMPACT') == '1'
scale = float(os.environ.get('WINIT_X11_SCALE_FACTOR', '1'))
w, h = (900, 760) if compact else (1280, 900)
def mouse(px, py):
    click(round(px*scale), round(py*scale))
def move(px, py):
    xt.XTestFakeMotionEvent(display,-1,round(px*scale),round(py*scale),0)
    x.XFlush(display)
    time.sleep(.15)
def shot(name):
    attr=WindowAttributes(); x.XGetWindowAttributes(display,window,C.byref(attr))
    ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))

with tempfile.TemporaryDirectory(prefix='arcade-mouse-') as tmp:
    state = Path(tmp)/'state'
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data')
    if os.environ.get('ARCADE_TEST_LIGHT') == '1':
        theme = Path(tmp)/'state/omarchy/current/theme'
        theme.mkdir(parents=True)
        (theme/'colors.toml').write_text('background = "#f1eee4"\nforeground = "#272b27"\naccent = "#536e46"\n')
    app = subprocess.Popen([binary]+(['--compact'] if compact else []), env=env)
    try:
        for _ in range(160):
            found = windows()
            if found and mapped(found[0]): break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found) == 1, found
        window = found[0]
        x.XSetInputFocus(display, window, 1, 0); x.XFlush(display)
        time.sleep(1)
        def ready(name):
            for _ in range(160):
                ptr=C.c_void_p(); x.XFetchName(display,window,C.byref(ptr))
                value=C.string_at(ptr).decode(errors='replace') if ptr.value else ''
                if ptr.value: x.XFree(ptr)
                if value == name:
                    time.sleep(.4)
                    assert windows() == [window]
                    return
                assert app.poll() is None
                time.sleep(.1)
            raise AssertionError(('Window title', name, value))
        pad = 20 if compact else 30
        right = w - 24 - pad
        rail_w = min(350, max(240, (w-48-pad*2)*.32))
        bottom = h - 32 - 46
        top = 144
        games = ['Circuit Pinball','Solitaire','Scram','Invaders','Chess','Stack','Snake','Bubble','Blast','2048','Shatter']
        def select(index):
            mouse(right-rail_w/2, top+(index+.5)*(bottom-top)/len(games))
        def play(name):
            mouse(right-rail_w-30-63, bottom-53)
            ready(name+' - Omarchy Arcade')
        def home():
            mouse(75, 25)
            ready('Omarchy Arcade')
        shot('shelf')
        # All eleven entries can be selected, launched and left without a key.
        for index,name in enumerate(games):
            select(index); play(name)
            if index == 0:
                time.sleep(2)
                mouse(75,25)
                shot('pinball-confirm')
                # Modal buttons sit below its centred explanatory text.
                mouse(w/2+125,h/2+35)
                ready('Circuit Pinball - Omarchy Arcade')
                shot('pinball-cancel')
                mouse(75,25)
                mouse(w/2-90,h/2+35)
                ready('Omarchy Arcade')
            else:
                home()
        # Invaders: resume with the mouse, steer and fire, then verify the saved state.
        select(3); play('Invaders')
        save = state/'omarchy-invaders/session.json'
        initial = json.loads(save.read_text())['game']
        # Use the board's visible Resume button (geometry is authored in game units).
        # Native capture below records the exact layout for review.
        shot('invaders-paused')
        # The Game menu is common at the left of the game toolbar.
        mouse(35,65); mouse(90,130)
        time.sleep(.2)
        move(w*.72,h*.7)
        xt.XTestFakeButtonEvent(display,1,1,0); x.XFlush(display); time.sleep(.3)
        xt.XTestFakeButtonEvent(display,1,0,0); x.XFlush(display)
        key(ord('p'))  # Freeze the evidence before screenshot and shelf navigation.
        shot('invaders-mouse')
        home()
        data=json.loads(save.read_text())
        assert data['game']['ship'] > 430, data['game']['ship']
        assert data['game']['tick'] > 0
        # Shots can already have struck a bunker by the time we save. Only firing
        # resets the cooldown; without firing it falls by exactly one dt per tick.
        elapsed = (data['game']['tick'] - initial['tick']) / 120
        assert data['game']['cooldown'] + elapsed > initial['cooldown'] + .1, \
            ('Held primary button did not fire', initial, data['game'])
        # Arrow input takes control back after mouse input.
        play('Invaders'); key(ord('p')); key(0xff51,hold=.5); home()
        later=json.loads((state/'omarchy-invaders/session.json').read_text())
        assert later['game']['ship'] < data['game']['ship']-50
        print('PASS: eleven mouse launches/returns, Pinball confirm/cancel, Invaders mouse and keyboard handoff')
        key(ord('q'),True); app.wait(timeout=10)
        assert app.returncode == 0
    except Exception:
        if app.poll() is None and windows():
            shot("failure")
        raise
    finally:
        if app.poll() is None:
            app.terminate(); app.wait(timeout=10)
        x.XCloseDisplay(display)
