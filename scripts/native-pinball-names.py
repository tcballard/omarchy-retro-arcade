"""Run under Xvfb: exercise actual host typing, score rename, save and restart."""
from pathlib import Path

# Reuse the collection's X11 input helpers, not its unrelated game test cases.
exec(compile((Path(__file__).resolve().parent / 'native-check.py').read_text().split(
    'with tempfile.TemporaryDirectory')[0], 'native-input-helpers', 'exec'))

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

    app,window=open_game()
    try:
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
        if os.environ.get('ARCADE_NAMES_CAPTURE'):
            subprocess.run(['magick','import','-window',str(window),
                            os.environ['ARCADE_NAMES_CAPTURE']],check=True)
        key(ord('a'),True)
        for ch in 'cancelled':key(ord(ch))
        click(460,522)  # Cancel in the fixture's fixed-position dialog.
        key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
        second=snapshot()
        assert second['0.Name']=='Riel St-AmAnd9',second
        assert second['1.Name']=='Runner up'
        assert [int(second[f'{i}.Score']) for i in range(5)]==scores
        print('PASS native host typing, mixed case, spaces, punctuation, Backspace, Undo, rename, save/restart and Cancel')
    finally:
        if app.poll() is None:
            app.terminate();app.wait(timeout=10)
x.XCloseDisplay(display)
