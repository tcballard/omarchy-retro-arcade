"""Live palette replacement and theme-file recovery; X11, not Wayland acceptance."""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
with tempfile.TemporaryDirectory(prefix='arcade-mines-theme-') as tmp:
    state = Path(tmp) / 'state'
    theme = state / 'omarchy/current/theme/colors.toml'
    theme.parent.mkdir(parents=True)
    theme.write_text('background="#171c1a"\nforeground="#e4e8df"\naccent="#b3cb92"')
    env = dict(os.environ, XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data')
    app = subprocess.Popen([binary, '--game', 'minesweeper'], env=env)
    try:
        for _ in range(100):
            found = windows()
            if found: break
            assert app.poll() is None
            time.sleep(.1)
        w = found[0]
        x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.4)
        key(0x20)
        save = state / 'omarchy-retro-arcade/minesweeper.json'
        before = json.loads(save.read_text())['game']['cells']
        def background(): return ImageGrab.grab(xdisplay=os.environ['DISPLAY']).convert('RGB').getpixel((10,100))
        assert background() == (23,28,26)
        replacement = theme.with_suffix('.new')
        replacement.write_text('background="#f3f0e7"\nforeground="#262b24"\naccent="#356f85"')
        replacement.replace(theme)
        for _ in range(40):
            if background() == (243,240,231): break
            time.sleep(.1)
        assert background() == (243,240,231), 'Active desktop palette did not reload'
        theme.write_text('incomplete theme replacement')
        time.sleep(1.3)
        assert background() == (243,240,231), 'Malformed theme discarded last valid palette'
        # Use an installed contrasting family to prove the running app reloads fontconfig.
        current_font = subprocess.check_output(['fc-match','-f','%{family}','monospace'],env=env,text=True)
        alternate = subprocess.check_output(['fc-match','-f','%{family}','serif'],env=env,text=True).split(',')[0]
        if alternate not in current_font:
            from xml.sax.saxutils import escape
            before_title = ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((180,70,500,105)).tobytes()
            fontconfig = Path(tmp) / 'config/fontconfig/fonts.conf'
            fontconfig.parent.mkdir(parents=True,exist_ok=True)
            fontconfig.write_text('<fontconfig><match target="pattern"><test name="family"><string>monospace</string></test><edit name="family" mode="prepend_first" binding="strong"><string>'+escape(alternate)+'</string></edit></match></fontconfig>')
            assert alternate in subprocess.check_output(['fc-match','-f','%{family}','monospace'],env=env,text=True)
            time.sleep(3.)
            after_title = ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((180,70,500,105)).tobytes()
            assert after_title != before_title, 'Desktop font did not reload in running game'
            print('PASS: running app adopts changed fontconfig selection.')
        else:
            print('SKIP: no contrasting installed font for live typography test.')
        key(ord('q'),True);app.wait(timeout=8)
        assert json.loads(save.read_text())['game']['cells'] == before
        print('PASS: live dark-to-light palette reload, invalid theme retention, unchanged board.')
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
