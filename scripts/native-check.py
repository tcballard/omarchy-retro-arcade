"""X11 integration harness; not an application/runtime dependency.
Run inside xvfb-run, with the built binary as argv[1].
"""
import ctypes as C
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

binary = str(Path(sys.argv[1]).resolve())
x = C.CDLL('libX11.so.6')
xt = C.CDLL('libXtst.so.6')
x.XOpenDisplay.argtypes=[C.c_char_p]; x.XOpenDisplay.restype=C.c_void_p
x.XDefaultRootWindow.argtypes=[C.c_void_p]; x.XDefaultRootWindow.restype=C.c_ulong
x.XQueryTree.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(C.c_ulong),C.POINTER(C.c_ulong),C.POINTER(C.POINTER(C.c_ulong)),C.POINTER(C.c_uint)]
x.XSetInputFocus.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_ulong]
x.XFetchName.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(C.c_void_p)]
x.XFree.argtypes=[C.c_void_p]
x.XKeysymToKeycode.argtypes=[C.c_void_p,C.c_ulong]; x.XKeysymToKeycode.restype=C.c_uint
x.XFlush.argtypes=[C.c_void_p]
x.XCloseDisplay.argtypes=[C.c_void_p]
class WindowAttributes(C.Structure):
    _fields_ = [(name, C.c_int) for name in ('x', 'y', 'width', 'height', 'border_width', 'depth')] + [
        ('visual', C.c_void_p), ('root', C.c_ulong), ('window_class', C.c_int),
        ('bit_gravity', C.c_int), ('win_gravity', C.c_int), ('backing_store', C.c_int),
        ('backing_planes', C.c_ulong), ('backing_pixel', C.c_ulong),
        ('save_under', C.c_int), ('colormap', C.c_ulong), ('map_installed', C.c_int),
        ('map_state', C.c_int), ('all_event_masks', C.c_long),
        ('your_event_mask', C.c_long), ('do_not_propagate_mask', C.c_long),
        ('override_redirect', C.c_int), ('screen', C.c_void_p)]
x.XGetWindowAttributes.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(WindowAttributes)]
xt.XTestFakeKeyEvent.argtypes=[C.c_void_p,C.c_uint,C.c_int,C.c_ulong]
xt.XTestFakeButtonEvent.argtypes=[C.c_void_p,C.c_uint,C.c_int,C.c_ulong]
xt.XTestFakeMotionEvent.argtypes=[C.c_void_p,C.c_int,C.c_int,C.c_int,C.c_ulong]
class WindowAttributes(C.Structure):
    _fields_=[('x',C.c_int),('y',C.c_int),('width',C.c_int),('height',C.c_int),('border_width',C.c_int),('depth',C.c_int),('visual',C.c_void_p),('root',C.c_ulong),('window_class',C.c_int),('bit_gravity',C.c_int),('win_gravity',C.c_int),('backing_store',C.c_int),('backing_planes',C.c_ulong),('backing_pixel',C.c_ulong),('save_under',C.c_int),('colormap',C.c_ulong),('map_installed',C.c_int),('map_state',C.c_int),('all_event_masks',C.c_long),('your_event_mask',C.c_long),('do_not_propagate_mask',C.c_long),('override_redirect',C.c_int),('screen',C.c_void_p)]
x.XGetWindowAttributes.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(WindowAttributes)]
def mapped(window):
    attributes=WindowAttributes()
    return x.XGetWindowAttributes(display,window,C.byref(attributes)) and attributes.map_state==2

display=x.XOpenDisplay(os.environ['DISPLAY'].encode()); assert display

def windows():
    root=C.c_ulong();parent=C.c_ulong();children=C.POINTER(C.c_ulong)();n=C.c_uint()
    x.XQueryTree(display,x.XDefaultRootWindow(display),C.byref(root),C.byref(parent),C.byref(children),C.byref(n))
    result=[]
    for i in range(n.value):
        name=C.c_void_p();x.XFetchName(display,children[i],C.byref(name))
        if name.value:
            text=C.string_at(name).decode(errors='replace');x.XFree(name)
            attributes=WindowAttributes()
            if ('Omarchy' in text and
                    x.XGetWindowAttributes(display,children[i],C.byref(attributes)) and
                    attributes.map_state==2):
                result.append(children[i])
    if children:x.XFree(children)
    return result

def key(sym,ctrl=False,hold=.06):
    control=x.XKeysymToKeycode(display,0xffe3);code=x.XKeysymToKeycode(display,sym)
    if ctrl:xt.XTestFakeKeyEvent(display,control,1,0)
    xt.XTestFakeKeyEvent(display,code,1,0);x.XFlush(display);time.sleep(hold)
    xt.XTestFakeKeyEvent(display,code,0,0)
    if ctrl:xt.XTestFakeKeyEvent(display,control,0,0)
    x.XFlush(display);time.sleep(.18)

def click(px,py):
    xt.XTestFakeMotionEvent(display,-1,px,py,0);x.XFlush(display);time.sleep(.1)
    xt.XTestFakeButtonEvent(display,1,1,0);x.XFlush(display);time.sleep(.06)
    xt.XTestFakeButtonEvent(display,1,0,0);x.XFlush(display);time.sleep(.2)

with tempfile.TemporaryDirectory(prefix='arcade-native-') as tmp:
    state=Path(tmp)/'state'
    env=dict(os.environ,XDG_STATE_HOME=str(state),XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data')
    app=subprocess.Popen([binary]+(['--compact'] if os.environ.get('ARCADE_TEST_COMPACT')=='1' else []),env=env)
    try:
        for _ in range(100):
            found=windows()
            if found and mapped(found[0]):break
            assert app.poll() is None
            time.sleep(.1)
        assert len(found)==1,found
        window=found[0];time.sleep(.6);x.XSetInputFocus(display,window,1,0);x.XFlush(display);time.sleep(.3)
        duplicate=subprocess.run([binary],env=env,capture_output=True,timeout=5)
        assert duplicate.returncode!=0 and b'already running' in duplicate.stderr
        def ready(name):
            for _ in range(160):
                ptr=C.c_void_p();x.XFetchName(display,window,C.byref(ptr))
                value=C.string_at(ptr).decode(errors='replace') if ptr.value else ''
                if ptr.value:x.XFree(ptr)
                if value==name:
                    time.sleep(.3)
                    assert windows()==[window]
                    return
                assert app.poll() is None
                time.sleep(.1)
            raise AssertionError(('Window never became ready',name,value))
        def enter(name):
            key(0xff0d);ready(name+' - Omarchy Arcade')
        def home():
            key(ord('h'),True);ready('Omarchy Arcade')
        # Enter Solitaire from the shared shelf, draw, leave and reopen.
        key(0xff53);enter('Solitaire');key(0x20);home()
        save=state/'omarchy-solitaire/session.json'
        first=json.loads(save.read_text());assert first['game']['state']['moves']==1,first
        enter('Solitaire');home()
        second=json.loads(save.read_text());assert second['game']==first['game']
        # Scram and Invaders use their existing storage identities.
        key(0xff53);enter('Scram');key(0xff53,hold=.5);home()
        scram=json.loads((state/'omarchy-munch/session.json').read_text());assert scram['version']>=1
        key(0xff53);enter('Invaders');key(0x20,hold=.4);home()
        invaders=json.loads((state/'omarchy-invaders/session.json').read_text());assert invaders['version']==1
        key(0xff53);enter('Chess')
        # Locate the real board from its theme colours; independent of window size and chrome.
        from PIL import ImageGrab
        im=ImageGrab.grab(xdisplay=os.environ['DISPLAY']).convert('RGB')
        coords=[(i%im.width,i//im.width) for i,c in enumerate(im.getdata()) if c in [(216,219,212),(107,116,96)]]
        assert coords,'Chess board did not render'
        left=min(c[0] for c in coords);right=max(c[0] for c in coords)+1
        top=min(c[1] for c in coords);bottom=max(c[1] for c in coords)+1
        click(int(left+(right-left)*4.5/8),int(top+(bottom-top)*6.5/8))
        click(int(left+(right-left)*4.5/8),int(top+(bottom-top)*4.5/8))
        chess_path=state/'omarchy-chess/session.json'
        for _ in range(100):
            if chess_path.exists() and len(json.loads(chess_path.read_text())['moves'])>=2:break
            time.sleep(.1)
        assert len(json.loads(chess_path.read_text())['moves'])>=2, 'Stockfish did not answer the native move'
        home()
        assert (state/'omarchy-chess/session.json').is_file()
        # Stack is another embedded game with a separate resumable run.
        key(0xff53);enter('Stack');key(0xff0d);key(0xff53);key(0x20);home()
        stack=json.loads((state/'omarchy-stack/session.json').read_text())
        assert stack['marathon']['locks']>=1
        # Snake is appended to the shelf; preserve the original five-game sequence.
        key(0xff53);enter('Snake');home()
        # Pinball runs within the SAME native window; no SDL desktop window.
        key(0xff53);enter('Bubble');key(0xff0d);key(0x20);home()
        key(0xff53);enter('Blast');home()
        key(0xff53);enter('2048');key(0xff51);home()
        assert (state/'omarchy-retro-arcade/2048.json').is_file()
        key(0xff53);enter('Shatter');home()
        assert (state/'omarchy-retro-arcade/shatter.json').is_file()
        key(0xff53);enter('Circuit Pinball');time.sleep(.7)
        assert windows()==[window],windows()
        key(0x20,hold=.6);key(ord('a'),hold=.2);key(ord('d'),hold=.2)
        key(ord('h'),True);time.sleep(.2)
        # Confirm return using the first modal action; keyboard focus is explicit.
        key(0xff0d);ready('Omarchy Arcade')
        time.sleep(.4)
        assert windows()==[window]
        key(0xff53);enter('Solitaire');home()
        # Close from the app-level shortcut.
        key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
        assert json.loads(save.read_text())['game']==first['game']
        print('PASS: singleton; one window across eleven games; Solitaire draw/save/reopen; legacy save paths; Stockfish replies to native move; native keys; clean shutdown.')
    finally:
        if app.poll() is None:app.kill();app.wait()
x.XCloseDisplay(display)
