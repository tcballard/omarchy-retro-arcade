"""FreeSki native X11 input, lifecycle, save and layout evidence.
Run under xvfb-run; uses normal controls and the public save file, no game hooks.
Usage: native-freeski.py BINARY OUTPUT [dark|light|compact|200]
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
    def wait_phase(phase, action):
        deadline=time.monotonic()+5
        while True:
            observed=read()
            if observed['run']['phase']==phase:return observed
            assert app.poll() is None, (action, variant, 'app exited', app.returncode)
            assert time.monotonic()<deadline, (action, variant, observed['run'])
            time.sleep(.05)
    def capture(name):
        attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))
    def move(px,py):
        xt.XTestFakeMotionEvent(display,-1,int(px*scale),int(py*scale),0);x.XFlush(display);time.sleep(.1)
    def press(button,down):xt.XTestFakeButtonEvent(display,button,int(down),0);x.XFlush(display)
    app,w=launch()
    try:
        assert read()['run']['phase']=='Ready';capture('ready')
        # Start with the ordinary visible button; test input handover on the open snow
        # before the faster run reaches trees. The toolbar has a fixed logical origin.
        click(460*scale,96*scale);time.sleep(2.3)
        key(ord('d'),hold=.25);capture('skiing')
        for _ in range(40):
            if read()['run']['phase']=='Running':break
            time.sleep(.05)
        assert read()['run']['phase']=='Running'
        # A held F can auto-repeat as extra presses and toggle fast mode back off.
        key(ord('f'))
        for _ in range(40):
            if read()['run']['fast_mode']:break
            time.sleep(.05)
        assert read()['run']['fast_mode'];capture('fast-mode')
        key(0xff1b);paused=read();assert paused['run']['phase']=='Paused';assert paused['run']['distance']>10
        assert paused['run']['fast_mode']
        capture('paused');time.sleep(.3);assert read()==paused
        key(ord('f'));assert read()==paused
        key(ord('a'),hold=.1);assert read()==paused
        # Returning focus must not resume, even with a held steering key.
        key(0xff0d);time.sleep(.2)
        click(365*scale,96*scale);assert not read()['run']['fast_mode']
        x.XSetInputFocus(display,x.XDefaultRootWindow(display),1,0);x.XFlush(display);time.sleep(.3)
        unfocused=read();assert unfocused['run']['phase']=='Paused'
        x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.3);assert read()==unfocused
        key(0xff0d)
        # Mouse aim changes heading through the same engine, then fresh keyboard takes over.
        width=900 if variant in ['compact','200'] else 1280
        move(width/2-100,450);time.sleep(.25);key(0xff1b)
        assert read()['run']['heading']<0
        key(0xff0d);key(ord('d'),hold=1.1);key(0xff1b)
        handover=read()
        assert handover['run']['heading']>0, ('keyboard handover', variant, handover['run'])
        # Observe both transitions: an old Paused checkpoint must not satisfy
        # the F1 wait, and a delayed F1 must not leave a Running snapshot.
        key(0xff0d);wait_phase('Running', 'resume before help')
        key(0xffbe);help_save=wait_phase('Paused', 'open help')
        capture('help');key(ord('d'))
        assert read()==help_save, ('help steering isolation', variant, help_save, read())
        key(0xff1b);assert read()==help_save
        # Same window and exact suspended attempt survive shelf exit and normal close/reopen.
        saved=read();key(ord('h'),True);assert windows()==[w];capture('shelf')
        key(0xff0d);assert windows()==[w];assert read()==saved
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        app,w=launch();assert read()==saved
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        # Optional fixtures are generated by the production-engine evidence example.
        fixtures=os.environ.get('FREESKI_FIXTURES')
        if fixtures:
            for fixture in ['mid-jump','recovery','free-long-run','free-mid-jump','free-recovery','chase-warning','chase-active','chase-recovery','chase-caught','slalom-progress','slalom-finished','slalom-cup-finished']:
                original=json.loads((Path(fixtures)/(fixture+'.json')).read_text())
                save.write_text(json.dumps(original))
                app,w=launch();loaded=read()
                expected=dict(original)
                expected['run']=dict(original['run'],phase='Paused') if original['run']['phase']=='Running' else original['run']
                assert loaded==expected, (fixture,loaded,expected)
                capture(fixture)
                time.sleep(.3);assert read()==loaded
                if fixture == 'chase-active':
                    before=loaded
                    key(0xff0d);key(ord('f'));time.sleep(.8);key(0xff1b)
                    loaded=read()
                    assert loaded['run']['phase']=='Paused'
                    assert loaded['run']['fast_mode']
                    assert loaded['run']['distance']>before['run']['distance']
                    assert loaded['chase']['position']!=before['chase']['position']
                    capture('chase-fast')
                    key(ord('h'),True);key(0xff0d);assert read()==loaded
                key(ord('q'),True);app.wait(timeout=8)
                assert read()==loaded
        save.unlink()  # Only this script's disposable state, after closing the app.
        app,w=launch();click(460*scale,96*scale)
        # Assert a simulation outcome, not that 1.1 seconds of CI wall time
        # delivered enough rendered input frames. Starting on fresh snow also
        # keeps unrelated terrain collisions out of this steering check.
        deadline=time.monotonic()+5
        while read()['run']['phase']=='Ready' and time.monotonic()<deadline:
            time.sleep(.05)
        assert read()['run']['phase']=='Running', ('quarter-turn start', variant, read()['run'])
        code=x.XKeysymToKeycode(display,ord('d'))
        xt.XTestFakeKeyEvent(display,code,1,0);x.XFlush(display)
        try:
            # Active runs persist every 300 ticks. Wait for that public evidence;
            # do not synthesize additional resumes or relax the exact heading.
            deadline=time.monotonic()+12
            while True:
                observed=read()['run']
                assert observed['phase']=='Running', ('quarter-turn interrupted', variant, observed)
                if abs(observed['heading']-3.141592653589793/2)<1e-10:
                    break
                assert time.monotonic()<deadline, ('quarter-turn timeout', variant, observed)
                time.sleep(.05)
        finally:
            xt.XTestFakeKeyEvent(display,code,0,0);x.XFlush(display)
        key(0xff1b);turned=read();capture('quarter-turn')
        assert turned['run']['phase']=='Paused', ('quarter-turn pause', variant, turned['run'])
        assert abs(turned['run']['heading']-3.141592653589793/2)<1e-10, ('quarter-turn heading', variant, turned['run'])
        key(0xff0d);time.sleep(.3);key(0xff1b);released=read()
        assert released['run']['heading']==turned['run']['heading']
        assert abs(released['run']['position']['y']-turned['run']['position']['y'])<1e-10
        key(ord('q'),True);app.wait(timeout=8)
        save.unlink()
        app,w=launch();capture('mode-choice')
        click(260*scale,96*scale);assert read()['mode']=='FreeSki';capture('free-ready')
        click(460*scale,96*scale);time.sleep(3.);capture('free-skiing');key(0xff1b)
        endless_save=read();assert endless_save['mode']=='FreeSki'
        assert endless_save['run']['phase']=='Paused'
        key(ord('h'),True);assert windows()==[w]
        key(0xff0d);assert read()==endless_save
        key(ord('q'),True);app.wait(timeout=8)
        app,w=launch();assert read()==endless_save
        key(ord('q'),True);app.wait(timeout=8)
        save.unlink()
        app,w=launch()
        click(260*scale,96*scale)
        # Click the label interior and wait for the UI's persisted result; the
        # small checkbox and a fixed 200 ms delay proved flaky on compact CI.
        click(450*scale,130*scale)
        for _ in range(40):
            if read()['chase_enabled']:break
            time.sleep(.05)
        assert read()['chase_enabled'];capture('chase-ready')
        key(ord('m'),True);assert read()['muted']
        key(0xff0d);time.sleep(.5);key(0xff1b)
        chase_ready=read();assert chase_ready['run']['phase']=='Paused'
        key(ord('q'),True);app.wait(timeout=8)
        app,w=launch();assert read()==chase_ready
        key(ord('q'),True);app.wait(timeout=8)
        save.unlink()
        app,w=launch();click(338*scale,96*scale)
        assert read()['mode']=='Slalom';assert read()['unlocked_courses']==1;capture('slalom-ready')
        key(0xff0d);time.sleep(3.);capture('slalom-skiing');key(0xff1b)
        slalom=read();assert slalom['run']['phase']=='Paused';assert slalom['run']['ticks']>100
        key(ord('q'),True);app.wait(timeout=8)
        app,w=launch();assert read()==slalom
        key(ord('q'),True);app.wait(timeout=8)
        save.write_text('future-save-do-not-replace')
        app,w=launch();capture('save-error');key(0xff0d);key(ord('q'),True);app.wait(timeout=8)
        assert save.read_text()=='future-save-do-not-replace'
        print(f'PASS: FreeSki {variant}: visible start, steering handover, pause/focus/help isolation, one-window switching, reopen and invalid-save retention.',flush=True)
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
