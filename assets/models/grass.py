"""Build the grass model in a fresh Blender scene."""
from _common import *

def build(context):
    clear()
    for i in range(7):a=i*2.4;rod('Grass blade',(0,0,0),(math.cos(a)*.26,math.sin(a)*.26,.3+i%3*.1),.035,leaf[i%4])
