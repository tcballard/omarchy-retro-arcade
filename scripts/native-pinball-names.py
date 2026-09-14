"""Run under Xvfb: exercise host typing, Ctrl-held clipboard paste and restart."""
from pathlib import Path
import select

# Reuse the collection's X11 input helpers, not its unrelated game test cases.
exec(compile((Path(__file__).resolve().parent / 'native-check.py').read_text().split(
    'with tempfile.TemporaryDirectory')[0], 'native-input-helpers', 'exec'))

# SDL serves the real X11 clipboard from a separate process, just as another
# desktop app would. No clipboard utility or changes to the user's desktop.
clipboard_source = r'''
import ctypes as C, sys, time
sdl = C.CDLL('libSDL2-2.0.so.0')
sdl.SDL_SetClipboardText.argtypes = [C.c_char_p]
assert sdl.SDL_Init(0x20) == 0
assert sdl.SDL_SetClipboardText(sys.argv[1].encode()) == 0
print('ready', flush=True)
while True:
    sdl.SDL_PumpEvents()
    time.sleep(.01)
'''

with tempfile.TemporaryDirectory(prefix='arcade-name-test-') as tmp:
    env = dict(os.environ, XDG_DATA_HOME=tmp+'/data', XDG_CONFIG_HOME=tmp+'/config',
               XDG_STATE_HOME=tmp+'/state', SDL_AUDIODRIVER='dummy', WINIT_X11_SCALE_FACTOR='1')
    env.pop('WAYLAND_DISPLAY', None)
    saved = Path(tmp)/'data/omarchy-spacecadet/circuit/imgui_pb.ini'
    saved.parent.mkdir(parents=True)
    names = ['Player 1', 'Runner up', '', '', '']
    scores = [1000, 500, -999, -999, -999]
    checksum = sum(scores) + sum(sum(name.encode()) for name in names)
    saved.write_text('[Window][High Scores]\nPos=350,250\nSize=400,230\nCollapsed=0\n\n'
                     '[Pinball][Settings]\nLanguage=en\n'+''.join(
                         f'{i}.Name={name}\n{i}.Score={score}\n'
                         for i,(name,score) in enumerate(zip(names,scores)))+
                     f'Verification={checksum}\n')

    def snapshot():
        return dict(line.split('=',1) for line in saved.read_text().splitlines() if '=' in line)

    def assert_saved(expected):
        current=snapshot()
        assert current['0.Name']==expected,current
        assert current['1.Name']=='Runner up',current
        assert [int(current[f'{i}.Score']) for i in range(5)]==scores,current
        # The legacy checksum uses signed char bytes on the x86_64 CI builds.
        checksum=sum(scores)+sum(byte if byte<128 else byte-256
                                for i in range(5) for byte in current[f'{i}.Name'].encode())
        assert int(current['Verification'])==checksum,current

    def open_game():
        app = subprocess.Popen([binary, '--game', 'pinball'], env=env)
        for _ in range(100):
            found=windows()
            if found and mapped(found[0]):
                break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found)==1,found
        window=found[0]
        x.XSetInputFocus(display,window,1,0);x.XFlush(display);time.sleep(1)
        return app,window

    def high_scores():
        # Fixed 1280x900 host: Game menu, then the High Scores item.
        click(35,57)
        if os.environ.get('ARCADE_NAMES_CAPTURE'):
            subprocess.run(['magick','import','-window',str(window),
                            os.environ['ARCADE_NAMES_CAPTURE']+'.menu.png'],check=True)
        click(65,168);time.sleep(.3)

    pasted="Zoë O'Neil 7!"
    clipboard=subprocess.Popen([sys.executable,'-u','-c',clipboard_source,pasted],
                               env=dict(env,SDL_VIDEODRIVER='x11'),stdout=subprocess.PIPE,
                               text=True)
    app=None
    try:
        assert select.select([clipboard.stdout],[],[],10)[0], 'Clipboard did not start'
        assert clipboard.stdout.readline().strip()=='ready'
        app,window=open_game()
        high_scores()
        if os.environ.get('ARCADE_NAMES_CAPTURE'):
            subprocess.run(['magick','import','-window',str(window),
                            os.environ['ARCADE_NAMES_CAPTURE']],check=True)
        key(ord('a'),True)
        for ch in 'Riel St-AmAnd9':
            # XTest key symbols ignore case; explicitly hold Shift for capitals.
            shift=ch.isupper()
            if shift: xt.XTestFakeKeyEvent(display,x.XKeysymToKeycode(display,0xffe1),1,0)
            key(ord(ch.lower()))
            if shift: xt.XTestFakeKeyEvent(display,x.XKeysymToKeycode(display,0xffe1),0,0)
            x.XFlush(display)
        key(0xff08)  # Backspace removes the final 9; Undo restores it.
        key(ord('z'),True)
        key(0xff0d)
        first=snapshot()
        assert first['0.Name']=='Riel St-AmAnd9',first
        assert [int(first[f'{i}.Score']) for i in range(5)]==scores
        key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
        app,window=open_game();high_scores()
        # Select the existing name, paste via the host's real clipboard event,
        # and save BEFORE releasing Ctrl. A deferred-until-release fix must fail.
        ctrl=x.XKeysymToKeycode(display,0xffe3)
        xt.XTestFakeKeyEvent(display,ctrl,1,0);x.XFlush(display)
        try:
            key(ord('a'))
            key(ord('v'));time.sleep(.5)
            click(405,522)  # OK, while Ctrl remains physically held.
            assert_saved(pasted)
        finally:
            xt.XTestFakeKeyEvent(display,ctrl,0,0);x.XFlush(display)
        key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
        app,window=open_game();high_scores()
        # Submit the name loaded by the restarted engine, not merely the file
        # left by the previous process. This also exercises checksum validation.
        key(0xff0d);assert_saved(pasted)
        high_scores()
        if os.environ.get('ARCADE_NAMES_CAPTURE'):
            subprocess.run(['magick','import','-window',str(window),
                            os.environ['ARCADE_NAMES_CAPTURE']],check=True)
        key(ord('a'),True)
        for ch in 'cancelled':key(ord(ch))
        click(460,522)  # Cancel in the fixture's fixed-position dialog.
        key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
        assert_saved(pasted)
        print('PASS native host typing, Backspace, Undo, Ctrl-held UTF-8 clipboard paste over an existing name, save before Ctrl release, restart, checksum, unchanged scores and Cancel')
    finally:
        clipboard.kill();clipboard.wait(timeout=10)
        if app is not None and app.poll() is None:
            app.terminate();app.wait(timeout=10)
x.XCloseDisplay(display)
