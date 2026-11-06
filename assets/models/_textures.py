"""Batch texture hook. Preserve the original seed after both thicket builds."""
import bpy,random,math

OUTPUTS=("jungle_ground.png",)

def build(context):
    for _ in range(120):
        random.random()
    size=256;image=bpy.data.images.new('Painted jungle ground',width=size,height=size);pixels=[]
    for y in range(size):
     for x in range(size):
      grain=random.uniform(-.035,.035);wave=math.sin(x/size*math.tau*7)*math.sin(y/size*math.tau*5)*.025
      blade=.10 if random.random()<.025 else 0
      pixels.extend((.82+grain+wave+blade,.87+grain+wave+blade,.67+grain+wave,1))
    image.pixels.foreach_set(pixels);out=context.texture_directory;out.mkdir(parents=True,exist_ok=True);image.filepath_raw=str(out/'jungle_ground.png');image.file_format='PNG';image.save()
