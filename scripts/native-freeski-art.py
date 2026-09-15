"""FreeSki artwork captures from production fixtures and ordinary native controls.
Run under xvfb-run; uses normal controls and the public save file, no game hooks.
Requires FREESKI_FIXTURES from the three evidence examples.
Usage: native-freeski-art.py BINARY OUTPUT [dark|light|compact|200]
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
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))
    fixtures=Path(os.environ['FREESKI_FIXTURES'])
    save.parent.mkdir(parents=True,exist_ok=True)
    app=None
    def hold(symbol,down):
        xt.XTestFakeKeyEvent(display,x.XKeysymToKeycode(display,symbol),int(down),0);x.XFlush(display)
    try:
        for name in ['ramp-approach','free-long-run','mid-jump','recovery','slalom-progress','slalom-finished']:
            original=json.loads((fixtures/(name+'.json')).read_text())
            save.write_text(json.dumps(original))
            app,w=launch()
            expected=dict(original)
            if original['run']['phase']=='Running':expected['run']=dict(original['run'],phase='Paused')
            assert read()==expected
            capture(name+'-restored')
            if original['run']['phase']=='Running':
                key(0xff0d);time.sleep(.08);capture(name+'-live')
                if name=='mid-jump':
                    for frame in range(6):
                        time.sleep(.10);capture('landing-'+str(frame))
                if name=='free-long-run':
                    hold(ord('s'),True);time.sleep(.18);capture('braking');hold(ord('s'),False)
                    hold(ord('d'),True);time.sleep(.60);capture('right-turn');hold(ord('d'),False)
                key(0xff1b)
            suspended=read()
            key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
            app=None
            assert read()==suspended
        # Fresh, disposable practice: stationary and traverse profiles, shelf and settings.
        save.unlink()
        app,w=launch();capture('ready')
        key(0xff0d);key(ord('d'),hold=1.2);capture('traverse')
        key(0xff1b);key(ord('h'),True);capture('shelf')
        key(0xff0d);key(ord(','),True);time.sleep(.3)
        capture('settings')
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        click(int(attr.width/2-213*scale),int(attr.height/2+34*scale))
        assert read()['reduced_effects'] is True
        key(0xff1b);key(0xff0d);key(ord('a'),hold=1.1);time.sleep(.2);capture('reduced-effects')
        key(0xff1b);key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        app=None
        print('PASS: artwork fixture restoration, live captures and normal close:',variant,flush=True)
    finally:
        if app is not None and app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
