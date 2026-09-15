"""FreeSki menu layout and normal-control evidence with isolated saves.
Run under xvfb-run; uses normal controls and the public save file, no game hooks.
Usage: native-freeski-menus.py BINARY OUTPUT [dark|light|compact|200]
"""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
variant=sys.argv[3] if len(sys.argv)>3 else 'dark'
scale=2 if variant=='200' else 1
with tempfile.TemporaryDirectory(prefix='arcade-freeski-') as tmp:
    env=dict(os.environ,XDG_STATE_HOME=tmp+'/state',XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',WINIT_X11_SCALE_FACTOR=str(scale))
    if variant=='light':
        theme=Path(tmp)/'state/omarchy/current/theme/colors.toml'
        theme.parent.mkdir(parents=True);theme.write_text('background = "#f3f0e7"\nforeground = "#262b24"\naccent = "#526f3a"\n')
    save=Path(tmp)/'state/omarchy-retro-arcade/freeski.json'
    args=[binary,'--game','freeski']+(['--compact'] if variant in ['compact','200'] else [])
    def launch():
        app=subprocess.Popen(args,env=env)
        for _ in range(100):
            found=windows()
            if found and mapped(found[0]):break
            assert app.poll() is None;time.sleep(.1)
        assert len(found)==1
        w=found[0];time.sleep(.5);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.3)
        return app,w
    def read():return json.loads(save.read_text())
    def capture(name):
        time.sleep(.3)  # Allow the requested overlay to paint before capturing it.
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))
    app,w=launch()
    try:
        capture('practice-ready')
        key(ord('f'));assert read()['mode']=='FreeSki';capture('free-ready')
        key(ord('l'));assert read()['mode']=='Slalom';capture('slalom-ready')
        key(ord('p'))
        click(460*scale,96*scale);time.sleep(.5);assert read()['run']['phase']=='Running'
        key(0xff1b);paused=read();capture('paused')
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        click(attr.x+attr.width//2,attr.y+attr.height//2+57*scale)
        capture('restart');key(0xff1b);assert read()==paused
        key(ord(','),True);capture('settings');key(0xff1b);assert read()==paused
        key(0xffbe);capture('help');key(0xff1b);assert read()==paused
        key(ord('h'),True);assert windows()==[w];key(0xff0d);assert read()==paused
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        fixtures=os.environ.get('FREESKI_FIXTURES')
        if fixtures:
            for fixture in ['chase-caught','slalom-finished','slalom-cup-finished']:
                original=json.loads((Path(fixtures)/(fixture+'.json')).read_text())
                save.write_text(json.dumps(original));app,w=launch()
                assert read()==original;capture(fixture)
                key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
                assert read()==original
        save.write_text('future-save-do-not-replace')
        app,w=launch();capture('save-error');key(ord('q'),True);app.wait(timeout=8)
        assert save.read_text()=='future-save-do-not-replace'
        print(f'PASS: FreeSki menus {variant}: start, modes, pause, confirmation, settings, help, shelf, results and save retention.',flush=True)
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
