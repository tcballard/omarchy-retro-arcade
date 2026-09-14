"""Capture Tanks through native inputs; run under Xvfb, not Wayland acceptance."""
from pathlib import Path
from PIL import ImageGrab
exec(compile((Path(__file__).parent/'native-check.py').read_text().split('with tempfile.TemporaryDirectory')[0], 'arcade-input-helpers', 'exec'))
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
for variant in ['dark', 'light', 'compact', '200']:
 with tempfile.TemporaryDirectory(prefix='tanks-render-') as tmp:
  env = dict(os.environ, XDG_STATE_HOME=tmp+'/state', XDG_CONFIG_HOME=tmp+'/config', XDG_DATA_HOME=tmp+'/data', WINIT_X11_SCALE_FACTOR='2' if variant=='200' else '1')
  if variant=='light':
   t=Path(tmp)/'state/omarchy/current/theme/colors.toml';t.parent.mkdir(parents=True);t.write_text('background = "#f3f0e7"\nforeground = "#262b24"\naccent = "#526f3a"\n')
  app=subprocess.Popen([binary,'--game','tanks']+(['--compact'] if variant in ['compact','200'] else []),env=env)
  try:
   for _ in range(120):
    found=windows()
    if found and mapped(found[0]):break
    assert app.poll() is None;time.sleep(.1)
   assert len(found)==1
   w=found[0];time.sleep(.5);x.XSetInputFocus(display,w,1,0);x.XFlush(display);time.sleep(.2)
   def capture(name):
    attr=WindowAttributes();x.XGetWindowAttributes(display,w,C.byref(attr))
    ImageGrab.grab(xdisplay=os.environ['DISPLAY']).crop((attr.x,attr.y,attr.x+attr.width,attr.y+attr.height)).save(out/(variant+'-'+name+'.png'))
   capture('chooser')
   key(ord('m'),True);key(ord('3'));key(0xff0d);time.sleep(.2)
   capture('aim')
   key(0xff51,hold=2.6);key(ord('3'));key(0x20)
   save=Path(tmp)/'state/omarchy-retro-arcade/tanks.json'
   for _ in range(100):
    if save.exists() and json.loads(save.read_text()).get('resolution') is not None:break
    time.sleep(.03)
   else:raise AssertionError('Shot did not reach a saved impact')
   impact=json.loads(save.read_text());shooter=impact['resolution']['impact']['shooter']
   assert impact['game']['tanks'][shooter]['power']==1
   assert impact['resolution']['impact']['weapon']=='Digger'
   time.sleep(.12);capture('impact')
   time.sleep(1.0);capture('result')
   key(ord('q'),True);app.wait(timeout=8);assert app.returncode==0
   print('Captured',variant,flush=True)
  finally:
   if app.poll() is None:app.terminate();app.wait()
x.XCloseDisplay(display)
