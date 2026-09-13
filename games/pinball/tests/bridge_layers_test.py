"""The high bridge hides a ground ball while a raised ball remains visible."""
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

executable=str(Path(sys.argv[1]).resolve())
frames={}
for name,vector in [('empty','540 500 0 1 0 1'),('under','370 40 0 1 0 1'),('over','370 40 0 1 0 2')]:
    with tempfile.TemporaryDirectory() as tmp:
        shot=Path(tmp)/'frame.bmp'
        env=dict(os.environ,XDG_CONFIG_HOME=tmp,XDG_DATA_HOME=tmp,
                 SDL_VIDEODRIVER='dummy',SDL_AUDIODRIVER='dummy',
                 OMARCHY_TEST_TICKS='190',OMARCHY_TEST_SHOT='custom',
                 OMARCHY_TEST_VECTOR=vector,OMARCHY_TEST_SCREENSHOT=str(shot))
        p=subprocess.run([executable,'--omarchy-table','-sw'],env=env,capture_output=True,text=True,timeout=45)
        assert p.returncode==0,(p.stdout+p.stderr)[-2500:]
        data=shot.read_bytes()
        if len(sys.argv)>2:
            out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
            (out/(name+'.bmp')).write_bytes(data)
        offset=struct.unpack_from('<I',data,10)[0]
        w,h=struct.unpack_from('<ii',data,18)
        bits=struct.unpack_from('<H',data,28)[0]
        assert (w,abs(h))==(1152,790) and bits in (24,32)
        stride=((w*bits+31)//32)*4
        def pixel(x,y):
            row=h-1-y if h>0 else y
            pos=offset+row*stride+x*(bits//8)
            return tuple(data[pos:pos+3])
        frames[name]=[pixel(x,y) for y in range(38,70) for x in range(263,293)]
def changed(name):
    return sum(max(abs(a-b) for a,b in zip(base,pixel))>60 for base,pixel in zip(frames['empty'],frames[name]))
under,over=changed('under'),changed('over')
assert over>40, f'Raised ball is hidden by the deck: {over}'
assert under<over/3, f'Ground ball is drawn above the deck: under={under} over={over}'
print(f'PASS bridge layering: ground difference {under} pixels, raised ball {over} pixels')
