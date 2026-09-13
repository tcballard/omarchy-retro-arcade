"""Check tall/wide pinball sizing, paused state and pointer menus under Xvfb."""
from pathlib import Path
from PIL import ImageGrab, ImageChops
exec(compile((Path(__file__).resolve().parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'native-input-helpers', 'exec'))
x.XResizeWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_uint,C.c_uint]
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
with tempfile.TemporaryDirectory() as tmp:
 env=dict(os.environ,XDG_STATE_HOME=tmp+'/state',XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',WINIT_X11_SCALE_FACTOR='1')
 app=subprocess.Popen([binary,'--game','pinball'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 def capture():
  a=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(a))
  return ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((a.x,a.y,a.x+a.width,a.y+a.height)).convert('RGB')
 def resize(width,height):
  x.XResizeWindow(display,w,width,height);x.XFlush(display);time.sleep(1.5)
  assert windows()==[w] and app.poll() is None
  im=capture();assert im.size==(width,height);return im
 def changed(a,b):return ImageChops.difference(a,b).getbbox() is not None
 try:
  for _ in range(150):
   found=windows()
   if found:break
   assert app.poll() is None;time.sleep(.1)
  assert len(found)==1;w=found[0]
  x.XSetInputFocus(display,w,1,0);x.XFlush(display);resize(1152,836)
  # F2 starts a game; pause it before comparing state across target resizes.
  key(0xffbf);time.sleep(.8);key(ord('p'));time.sleep(.8)
  # Wait for the pause command and buffered worker frame to reach the window.
  # A fixed delay can compare one pre-pause frame on a slower native renderer.
  deadline=time.monotonic()+5
  paused=capture()
  while time.monotonic()<deadline:
   time.sleep(.3);current=capture()
   if not changed(paused.crop((0,75,1152,810)),current.crop((0,75,1152,810))):break
   paused=current
  paused.save(out/"paused-before.png");time.sleep(.4)
  settled=capture();settled.save(out/"paused-after.png")
  assert not changed(paused.crop((0,75,1152,810)),settled.crop((0,75,1152,810)))
  for width,height in [(941,1030),(900,1100),(1360,800),(900,760),(1152,836)]:
   im=resize(width,height);im.save(out/f'{width}x{height}.png')
   if height>width:
    amber=sum(r>140 and 60<g<230 and b<120 for r,g,b in im.crop((0,int(height*.84),width,height)).getdata())
    assert amber>100,'lower HUD is blank'
  assert not changed(paused.crop((0,75,1152,810)),capture().crop((0,75,1152,810))),'state changed during resize'
  resize(941,1030);before=capture();click(28,58);time.sleep(.5)
  assert changed(before.crop((0,65,300,350)),capture().crop((0,65,300,350))),'resized menu pointer failed'
  click(800,200);key(ord('q'),True);app.wait(timeout=10);assert app.returncode==0
  print('PASS native resizing, lower HUD, preserved pause/state and pointer menu')
 finally:
  if app.poll() is None:app.terminate();app.wait(timeout=10)
x.XCloseDisplay(display)
