"""Build the bush model in a fresh Blender scene."""
from _common import *

def build(context):
    clear()
    for i in range(9):a=i*2.4;ico('Shrub',(math.cos(a)*.35,math.sin(a)*.35,.3+i%3*.12),(.45,.27,.3),leaf[i%4],1)
