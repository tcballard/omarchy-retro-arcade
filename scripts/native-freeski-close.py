"""Capture real fast/ordinary poses and the chase preview from production saves.
Run under xvfb-run. Usage: BINARY OUTPUT FIXTURE [dark|light|compact|200]
The fixture comes from preview-evidence; all play and toggles use native keys.
"""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
fixture=Path(sys.argv[3]);variant=sys.argv[4] if len(sys.argv)>4 else 'dark'
with tempfile.TemporaryDirectory(prefix='freeski-close-') as tmp:
    state=Path(tmp)/'state'
    env=dict(os.environ,XDG_STATE_HOME=str(state),XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',WINIT_X11_SCALE_FACTOR='2' if variant=='200' else '1')
    palette=('#f3f0e7','#262b24','#526f3a') if variant=='light' else ('#171c1a','#e4e8df','#b3cb92')
    theme=state/'omarchy/current/theme/colors.toml';theme.parent.mkdir(parents=True)
    theme.write_text('background = "%s"\nforeground = "%s"\naccent = "%s"\n'%palette)
    save=state/'omarchy-retro-arcade/freeski.json';save.parent.mkdir(parents=True)
    original=json.loads(fixture.read_text());save.write_text(fixture.read_text())
    app=subprocess.Popen([binary,'--game','freeski']+(['--compact'] if variant in ['compact','200'] else []),env=env)
    try:
        for _ in range(100):
            found=windows()
            if found and mapped(found[0]):break
            assert app.poll() is None;time.sleep(.1)
        assert len(found)==1
        w=found[0];time.sleep(1.5 if variant=='200' else .4);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.2)
        def capture(name):
            attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
            ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(name+'.png'))
        def read():return json.loads(save.read_text())
        expected=dict(original,run=dict(original['run'],phase='Paused'))
        assert read()==expected
        capture('restored')
        key(0xff0d,hold=.01);time.sleep(.08);capture('fast-chase')
        key(0xff1b);paused=read()
        assert paused['run']['phase'] in ['Paused', 'Caught'], paused['run']['phase']
        assert paused['run']['fast_mode']
        assert paused['run']['ticks']>original['run']['ticks']
        assert paused['chase']['position']!=original['chase']['position']
        time.sleep(.2);assert read()==paused
        key(ord('h'),True);capture('shelf')
        assert read()==paused
        key(0xff0d);assert read()==paused
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        assert read()==paused
        # Reopen the exact suspended chase and verify normal shutdown saved it.
        app=subprocess.Popen([binary,'--game','freeski'],env=env)
        time.sleep(1);found=windows();assert len(found)==1
        w=found[0];x.XSetInputFocus(display,w,1,0);x.XFlush(display)
        assert read()==paused
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        print('PASS: live fast chase; actor movement; saved '+paused['run']['phase']+' state; shelf; close/reopen:',variant)
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
