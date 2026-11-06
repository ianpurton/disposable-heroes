"""Build the tree model in a fresh Blender scene."""
from _common import *

def build(context):
    clear();rod('Trunk',(0,0,0),(0,0,2.6),.24,wood)
    for i in range(9):a=i*2.4;ico('Broadleaf crown',(math.cos(a)*.8,math.sin(a)*.8,2.6+(i%3)*.35),(1.05,.85,.8),leaf[i%4],2)
