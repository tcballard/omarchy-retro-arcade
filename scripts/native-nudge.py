"""Native X/period/Up shake, release, pause/focus and tilt presentation."""
from pathlib import Path
from PIL import ImageGrab, ImageChops
exec(compile((Path(__file__).resolve().parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'native-input-helpers', 'exec'))
x.XResizeWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_uint,C.c_uint]
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory(prefix='arcade-nudge-') as tmp:
    env=dict(os.environ,XDG_STATE_HOME=tmp+'/state',XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',WINIT_X11_SCALE_FACTOR='1')
    with open(out/'runtime.log','w') as log:
        app=subprocess.Popen([binary,'--game','pinball'],env=env,stdout=log,stderr=subprocess.STDOUT)
        def edge(sym,down):
            xt.XTestFakeKeyEvent(display,x.XKeysymToKeycode(display,sym),int(down),0);x.XFlush(display)
        def capture():
            a=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(a))
            return ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((a.x,a.y,a.x+a.width,a.y+a.height)).convert('RGB')
        def board(im):return im.crop((220,120,400,220))
        def changed(a,b):return ImageChops.difference(a,b).getbbox() is not None
        def raised(im):
            # At this fixed window size the raised left blade crosses this
            # patch. Count its ivory face, not arbitrary pixel differences:
            # the rolling steel ball also crosses the old large flipper crop.
            pixels=im.crop((300,670,370,722)).convert('RGB').getdata()
            return sum(1 for r,g,b in pixels if r>215 and 0<=r-g<12 and 20<g-b<50)>100
        def wait_flipper(up):
            until=time.monotonic()+8
            while time.monotonic()<until:
                im=capture()
                if raised(im)==up:return im
                time.sleep(.05)
            im.save(out/'flipper-timeout.png')
            raise AssertionError('Flipper did not '+('rise' if up else 'return to rest'))
        def centred(reference):
            until=time.monotonic()+1.5
            while time.monotonic()<until:
                if not changed(board(reference),board(capture())):return True
                time.sleep(.03)
            return False
        try:
            for _ in range(150):
                found=windows()
                if found:break
                assert app.poll() is None;time.sleep(.1)
            assert len(found)==1;w=found[0]
            x.XResizeWindow(display,w,1152,836);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(2)
            rest=capture();edge(ord('a'),True);time.sleep(.4);held=capture();edge(ord('a'),False)
            if not changed(rest.crop((210,610,460,805)),held.crop((210,610,460,805))):
                key(ord('p'));time.sleep(.6)
            for sym,name in [(ord('x'),'left'),(ord('.'),'right'),(0xff52,'bottom')]:
                key(0xffbf);time.sleep(.8);rest=capture();edge(sym,True)
                until=time.monotonic()+3
                while True:
                    time.sleep(.02);held=capture()
                    if changed(board(rest),board(held)) or time.monotonic()>=until:break
                edge(sym,False)
                rest.save(out/(name+'-rest.png'));held.save(out/(name+'-shake.png'))
                assert changed(board(rest),board(held)),name+' has no visible shake'
                held.save(out/(name+'-shake.png'));time.sleep(.7)
                assert centred(rest),name+' shake did not release'
            print('PASS native X/period/Up shake and release',flush=True)
            key(0xffbf);time.sleep(1);rest=capture();edge(ord('x'),True);time.sleep(.15);key(ord('p'));edge(ord('x'),False);time.sleep(.15)
            assert centred(rest),'pause left table displaced'
            key(ord('p'));key(0xffbf);time.sleep(1)
            rest=capture();edge(ord('.'),True);time.sleep(.07)
            x.XSetInputFocus(display,x.XDefaultRootWindow(display),1,0);x.XFlush(display);edge(ord('.'),False);time.sleep(.15)
            assert centred(rest),'focus loss left table displaced'
            x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.1);key(ord('p'));time.sleep(1)
            print('PASS native pause/focus recentres table',flush=True)
            key(0xffbf);time.sleep(1)
            # New-game feed clears tilt after a simulation timer. Wait for its
            # ball to appear before holding a nudge, rather than starting while
            # the legacy slow-render loop is still waiting to feed the ball.
            def launcher(im):return im.crop((675,650,725,750))
            empty=launcher(capture());until=time.monotonic()+40
            while time.monotonic()<until:
                time.sleep(.2)
                if changed(empty,launcher(capture())):break
            else:raise AssertionError('New-game ball did not appear')
            # Positive control: prove that the native key and the blade
            # detector work before interpreting lack of movement as lockout.
            edge(ord('a'),True);wait_flipper(True).save(out/'flipper-raised.png')
            edge(ord('a'),False);wait_flipper(False)
            time.sleep(1);edge(ord('x'),True)
            # Observe lockout with a deadline under variable render timing.
            until=time.monotonic()+60;quiet=0
            while time.monotonic()<until and quiet<3:
                before=capture();edge(ord('a'),True);time.sleep(.4);held=capture();edge(ord('a'),False);time.sleep(.4)
                quiet=quiet+1 if not raised(before) and not raised(held) else 0
            edge(ord('x'),False);time.sleep(.3)
            assert quiet==3,'held nudge never locked the flipper'
            tilted=capture();tilted.save(out/'tilt.png')
            edge(ord('a'),True)
            try:
                until=time.monotonic()+1.5
                while time.monotonic()<until:
                    time.sleep(.05);blocked=capture()
                    if raised(blocked):
                        blocked.save(out/'tilt-held.png')
                        raise AssertionError('tilt did not disable flipper')
                blocked.save(out/'tilt-held.png')
            finally:edge(ord('a'),False)
            # A second F2 skips the original engine's startup light show.
            key(0xffbf);time.sleep(1);key(0xffbf);time.sleep(1)
            empty=launcher(capture());until=time.monotonic()+40
            while time.monotonic()<until:
                time.sleep(.2)
                if changed(empty,launcher(capture())):break
            else:raise AssertionError('Reset game did not feed a ball')
            reset=capture();reset.save(out/'reset.png');edge(ord('a'),True)
            wait_flipper(True).save(out/'reset-raised.png')
            edge(ord('a'),False)
            assert app.poll() is None
            key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
            print('PASS native tilt flipper lock and new-game recovery',flush=True)
        finally:
            if app.poll() is None:app.terminate();app.wait(timeout=10)
x.XCloseDisplay(display)
