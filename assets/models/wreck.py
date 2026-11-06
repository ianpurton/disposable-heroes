"""Build the wreck model in a fresh Blender scene."""
from _common import *

def build(context):
    clear()
    for i in range(12):o=cube('Charred debris',(random.uniform(-1.5,1.5),random.uniform(-1,1),random.uniform(.05,.35)),(random.uniform(.3,1.2),.15,.15),dark);o.rotation_euler.z=random.random()*math.tau
