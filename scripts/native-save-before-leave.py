"""Real-window save failure/retry, Ctrl+H, WM close and explicit discard."""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent / 'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)

def title(window):
    name=C.c_void_p();x.XFetchName(display,window,C.byref(name))
    result=C.string_at(name).decode() if name.value else ''
    if name.value:x.XFree(name)
    return result

def wait_for(predicate):
    deadline=time.monotonic()+8
    while not predicate():
        assert time.monotonic()<deadline, 'Timed out waiting for native state'
        time.sleep(.05)

# Deliver the same WM_DELETE_WINDOW event as the desktop close button.
class ClientMessage(C.Structure):
    _fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),
              ('display',C.c_void_p),('window',C.c_ulong),('message_type',C.c_ulong),
              ('format',C.c_int),('data',C.c_long*5)]
class XEvent(C.Union):
    _fields_=[('client',ClientMessage),('pad',C.c_long*24)]
x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XInternAtom.restype=C.c_ulong
x.XSendEvent.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_long,C.POINTER(XEvent)]
def close_window(window):
    event=XEvent();event.client.type=33;event.client.display=display
    event.client.window=window;event.client.format=32
    event.client.message_type=x.XInternAtom(display,b'WM_PROTOCOLS',0)
    event.client.data[0]=x.XInternAtom(display,b'WM_DELETE_WINDOW',0)
    assert x.XSendEvent(display,window,0,0,C.byref(event))
    x.XFlush(display);time.sleep(.3)

with tempfile.TemporaryDirectory(prefix='arcade-leave-') as tmp:
    state=Path(tmp)/'state';save=state/'omarchy-retro-arcade/shatter.json'
    env=dict(os.environ,XDG_STATE_HOME=str(state),XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data')
    app=subprocess.Popen([binary,'--game','shatter'],env=env)
    try:
        wait_for(lambda: bool(windows()))
        window=windows()[0];x.XSetInputFocus(display,window,1,0);x.XFlush(display);time.sleep(.4)
        key(ord('m'),True);key(0xff1b);wait_for(save.is_file)
        original=save.read_bytes();backup=save.with_suffix('.preserved');save.rename(backup);save.mkdir()
        key(ord('h'),True);time.sleep(.3)
        assert app.poll() is None and title(window).startswith('Shatter')
        ImageGrab.grab(xdisplay=os.environ['DISPLAY']).save(out/'save-failed.png')
        # Escape stays. Repeated WM close must neither exit nor silently discard.
        key(0xff1b);close_window(window);close_window(window)
        assert app.poll() is None and save.is_dir()
        assert backup.read_bytes()==original
        key(0xff1b)  # Stay again; retry a shelf transition with repaired storage.
        key(ord('h'),True);save.rmdir();key(0xff0d)
        wait_for(lambda: title(window)=='Omarchy Arcade')
        wait_for(save.is_file)
        assert json.loads(save.read_text())['campaign']==json.loads(original)['campaign']
        key(0xff0d);wait_for(lambda: title(window).startswith('Shatter'))
        # Failed close retains the game until the player explicitly discards.
        save.unlink();save.mkdir();close_window(window)
        assert app.poll() is None
        leave_anyway();app.wait(timeout=8);assert app.returncode==0
        assert save.is_dir() and backup.read_bytes()==original
        print('PASS: real save failure stays open, WM close cancellation, Stay, retry to shelf, explicit discard, original-byte preservation')
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
