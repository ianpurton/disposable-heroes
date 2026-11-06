"""Build the bunker model in a fresh Blender scene."""
from _common import *

def build(context):
    clear();cube('Concrete bunker',(0,0,.65),(3.2,2.4,1.3),sand,.15);cube('Bunker roof',(0,0,1.4),(3.5,2.7,.25),olive,.08);cube('Firing slit',(0,-1.21,.85),(1.8,.03,.25),dark,0);
