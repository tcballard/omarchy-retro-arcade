from pathlib import Path
from PIL import ImageGrab, ImageChops
exec(compile((Path(__file__).resolve().parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'native-input-helpers', 'exec'))
x.XResizeWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_uint,C.c_uint]
with tempfile.TemporaryDirectory() as tmp:
 env=dict(os.environ,XDG_STATE_HOME=tmp+'/state',XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',WINIT_X11_SCALE_FACTOR='1',OMARCHY_CHECK_GEOMETRY='1',OMARCHY_TRACE_CONTACTS='1')
 log=open(Path(tmp)/'engine.log','w+')
 app=subprocess.Popen([binary,'--game','pinball'],env=env,stdout=log,stderr=log)
 def edge(sym,down):
  xt.XTestFakeKeyEvent(display,x.XKeysymToKeycode(display,sym),int(down),0);x.XFlush(display);time.sleep(.4)
 def capture(right=False):
  a=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(a))
  im=ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((a.x,a.y,a.x+a.width,a.y+a.height))
  return im.crop((420 if right else 210,610,640 if right else 460,805))
 def changed(a,b):return ImageChops.difference(a,b).getbbox() is not None
 def stable(right=False):
  start=time.monotonic();last_change=start;previous=capture(right)
  while time.monotonic()-start<8:
   time.sleep(.15);current=capture(right)
   if changed(previous,current):last_change=time.monotonic()
   previous=current
   if time.monotonic()-start>1.2 and time.monotonic()-last_change>.6:return current
  raise AssertionError('Flipper did not settle')
 try:
  for _ in range(150):
   found=windows()
   if found:break
   assert app.poll() is None;time.sleep(.1)
  assert len(found)==1;w=found[0]
  x.XResizeWindow(display,w,1152,836);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(2)
  rest=stable();neutral=rest.copy();edge(ord('a'),True);held=stable();edge(ord('a'),False)
  if not changed(rest,held):key(ord('p'));time.sleep(.8)
  for classic,modern,right in [('z','a',False),('/','d',True)]:
   rest=stable(right);edge(ord(classic),True);held=stable(right);assert changed(rest,held)
   edge(ord(modern),True);edge(ord(classic),False);assert not changed(held,stable(right))
   edge(ord(modern),False);assert changed(held,stable(right))
  edge(ord('z'),True)
  x.XSetInputFocus(display,x.XDefaultRootWindow(display),1,0);x.XFlush(display);time.sleep(2);edge(ord('z'),False)
  x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(2);key(ord('p'));time.sleep(.4)
  until=time.monotonic()+15
  while changed(neutral,capture()) and time.monotonic()<until:time.sleep(.2)
  assert not changed(neutral,capture()),'Focus recovery did not return the flipper to rest'
  edge(ord('z'),True)
  until=time.monotonic()+8
  while not changed(neutral,capture()) and time.monotonic()<until:time.sleep(.2)
  assert changed(neutral,capture()),'Classic key failed after focus return'
  edge(ord('z'),False)
  # Play through normal input under variable native frame timing. Each restart
  # varies the bounce phase at which a fully charged launcher is released.
  for phase in (.2,.45,.7):
   key(0xffbf);time.sleep(phase)
   edge(ord(' '),True);time.sleep(1.2);edge(ord(' '),False)
   for _ in range(4):
    edge(ord('z'),True);edge(ord('/'),True)
    edge(ord('z'),False);edge(ord('/'),False)
  key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
  log.flush();log.seek(0);trace=log.read()
  assert 'NATIVE_GEOMETRY_CHECKS enabled' in trace, trace[-2500:]
  assert 'NATIVE_GEOMETRY_FAILURE' not in trace, trace[-2500:]
  contacts=[line.split() for line in trace.splitlines() if line.startswith('CONTACT ')]
  assert any(float(c[5])<800 and c[2]!='plunger' for c in contacts),'Native launches did not reach the playfield'
  log.close()
  print('PASS native classic aliases, overlap and focus recovery')
 finally:
  if app.poll() is None:app.terminate();app.wait(timeout=10)
x.XCloseDisplay(display)
