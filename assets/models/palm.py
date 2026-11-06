"""Build the palm model in a fresh Blender scene."""
from _common import *

def build(context):
    clear()
    for i in range(7):rod('Palm trunk',(math.sin(i*.14)*.3,0,i*.55),(math.sin((i+1)*.14)*.3,0,(i+1)*.55),.17,wood)
    for k in range(9):
     a=k*math.tau/9
     for j in range(4):
      r=.35+j*.5;o=ico('Palm frond',(math.cos(a)*r+.3,math.sin(a)*r,3.8+.3*math.sin(j)-j*.18),(.7,.22,.08),leaf[k%4],1);o.rotation_euler.z=a
