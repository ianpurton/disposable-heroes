"""Build the hut model in a fresh Blender scene."""
from _common import *

def build(context):
    clear()
    for x in [-1.5,1.5]:
     for y in [-1.1,1.1]:rod('Post',(x,y,0),(x,y,2.1),.13,wood)
    for x in [-1.45,1.45]:cube('Side wall',(x,0,1.),(.15,2.2,1.8),wood)
    cube('Back wall',(0,1.05,1.),(3.,.15,1.8),wood)
    for x in [-1.,1.]:cube('Front wall',(x,-1.05,1.),(.9,.15,1.8),wood)
    for x in [-.9,.9]:o=cube('Thatch roof',(x,0,2.15),(2.,2.8,.22),roof);o.rotation_euler.y=math.copysign(.42,x)
    for i in range(16):o=cube('Roof straw',(-1.75+i*.23,0,2.55-abs(-1.75+i*.23)*.43),(.12,2.85,.12),roof)
