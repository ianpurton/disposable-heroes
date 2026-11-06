"""Original synthesized effects; offline tooling, no runtime Python dependency."""
import wave,random,math,struct,pathlib
out=pathlib.Path(__file__).resolve().parents[1]/'assets/audio';out.mkdir(exist_ok=True)
rng=random.Random(1993)
for name,duration in [('rifle',.13),('explosion',1.1),('jungle',12.)]:
 samples=[];previous=0.
 for i in range(int(22050*duration)):
  t=i/22050;noise=rng.uniform(-1,1);previous=.94*previous+.06*noise
  if name=='rifle':v=(noise*.6+math.sin(t*900)*.25)*math.exp(-t*40)
  elif name=='explosion':v=(previous*3+noise*.18+math.sin(t*70)*.25)*math.exp(-t*4)
  else:v=previous*.12+math.sin(t*2400+math.sin(t*2)*40)*max(0,math.sin(t*1.8))**18*.025
  samples.append(struct.pack('<h',int(max(-1,min(1,v))*.65*32767)))
 with wave.open(str(out/(name+'.wav')),'wb') as f:f.setparams((1,2,22050,0,'NONE','not compressed'));f.writeframes(b''.join(samples))
