"""Actual native screenshots of all games. Run under Xvfb. No gameplay test hooks."""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
variant=sys.argv[3] if len(sys.argv)>3 else 'dark'
scale=2 if variant=='200' else 1
with tempfile.TemporaryDirectory(prefix='arcade-polish-') as tmp:
 env=dict(os.environ,XDG_STATE_HOME=tmp+'/state',XDG_CONFIG_HOME=tmp+'/config',XDG_DATA_HOME=tmp+'/data',WINIT_X11_SCALE_FACTOR=str(scale))
 if variant=='light':
  t=Path(tmp)/'state/omarchy/current/theme/colors.toml';t.parent.mkdir(parents=True);t.write_text('background = "#f3f0e7"\nforeground = "#262b24"\naccent = "#526f3a"\n')
 for game in ['shelf','pinball','solitaire','scram','invaders','chess','stack','snake','bubble','blast','2048','shatter','tanks','minesweeper']:
  args=[binary]+([] if game=='shelf' else ['--game',game])+(['--compact'] if variant in ['compact','200'] else [])
  app=subprocess.Popen(args,env=env)
  try:
   for _ in range(120):
    found=windows()
    if found:break
    assert app.poll() is None;time.sleep(.1)
   assert len(found)==1
   w=found[0];time.sleep(1.);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.2)
   if game=='stack':
    key(0xff0d);key(0xff51,hold=.3);key(0x20);key(0xff53,hold=.3);key(0x20);key(ord('c'));key(0xff52)
   elif game=='snake': key(ord('1'));key(0xff0d);key(0xff52,hold=.02)
   elif game in ['bubble','blast']: key(0xff0d)
   elif game=='scram':key(0xff53,hold=.25)
   elif game=='invaders':key(ord('p'));key(0x20,hold=.2)
   elif game=='pinball':key(0x20,hold=.5);time.sleep(2.)
   time.sleep(.2)
   attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
   ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(game+'.png'))
   assert windows()==[w]
   key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
   print('Captured',variant,game,flush=True)
  finally:
   if app.poll() is None:app.terminate();app.wait()
x.XCloseDisplay(display)
