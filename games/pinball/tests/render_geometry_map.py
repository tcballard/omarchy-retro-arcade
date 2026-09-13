"""Plot exported collision coordinates over the source art; never modifies the art."""
import json,sys,re
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
walls=json.loads(Path(sys.argv[1]).read_text())
base=Image.open(sys.argv[2]).convert('RGBA').crop((0,0,1024,1024))
for layer,label in [(1,'ground'),(2,'raised')]:
 overlay=Image.new('RGBA',base.size);d=ImageDraw.Draw(overlay)
 if layer==2:
  left=next(w['points'] for w in walls if w['name']=='ramp_rail_l')
  right=next(w['points'] for w in walls if w['name']=='ramp_rail_r')
  d.polygon([tuple(p) for p in left+right[::-1]],fill=(60,220,90,70))
 for w in walls:
  if not w['layers']&layer:continue
  pts=[tuple(p) for p in w['points']]
  ramp=w['name'].startswith('ramp_rail')
  if w['solid']:d.polygon(pts,fill=(255,50,130,80))
  d.line(pts,fill=(255,220,0,70) if ramp else (0,220,255,60),width=23)
  for x,y in pts:d.ellipse((x-11.25,y-11.25,x+11.25,y+11.25),fill=(255,220,0,50) if ramp else (0,220,255,40))
  d.line(pts,fill=(255,230,0,255) if ramp else (0,240,255,255),width=2)
 out=Image.alpha_composite(base,overlay).convert('RGB');d=ImageDraw.Draw(out)
 d.rectangle((5,5,212,48),fill='black');d.text((12,12),f'{label.upper()} collision map\nLines: walls. Bands: ball clearance.',fill='white')
 out.save(f'{sys.argv[3]}-{label}.png')

# Optional independent route-fixture overlay and legend, for layout review.
if len(sys.argv)>4:
 routes=[]
 for line in Path(sys.argv[4]).read_text().splitlines():
  match=re.match(r'\s*\{"([^\"]+)", \{(.*)\}\},',line)
  if match:
   points=[tuple(map(int,p)) for p in re.findall(r'\{(\d+),(\d+)\}',match[2])]
   routes.append((match[1],points))
 canvas=Image.new('RGB',(1450,1024),(14,20,24))
 canvas.paste(Image.open(f'{sys.argv[3]}-ground.png'),(0,0))
 d=ImageDraw.Draw(canvas)
 try:
  font=ImageFont.truetype('DejaVuSans.ttf',17)
  small=ImageFont.truetype('DejaVuSans.ttf',13)
 except OSError:font=small=ImageFont.load_default()
 d.text((1050,20),'VERIFIED GROUND ROUTES',font=font,fill='white')
 d.text((1050,52),'Ball diameter: 22.5 artwork pixels',font=small,fill='#bcc9d2')
 for index,(name,points) in enumerate(routes,1):
  d.line(points,fill='#6dffff',width=3)
  x,y=points[0];d.ellipse((x-10,y-10,x+10,y+10),fill='#092426',outline='#6dffff')
  d.text((x,y),str(index),font=small,fill='white',anchor='mm')
  d.text((1050,78+index*37),f'{index:2}. {name}',font=font,fill='#6dffff')
 d.text((1050,810),'Bands show ball-radius clearance.\nPink areas are solid obstacles.\nThe high bridge allows a ground underpass;\nits raised tube stays bounded separately.\n\nRoutes are tested against installed colliders.\nThey are not scripted ball movement.',font=small,fill='#bcc9d2',spacing=7)
 canvas.save(f'{sys.argv[3]}-routes.png')
