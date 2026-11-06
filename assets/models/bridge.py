"""Build the bridge model in a fresh Blender scene."""
from _common import *

def build(context):
    clear()
    for i in range(10):cube('Bridge plank',(0,-2.+i*.44,.1),(2.,.4,.2),wood)
    for x in [-.9,.9]:rod('Rail',(x,-2.,.9),(x,2.,.9),.07,wood)
