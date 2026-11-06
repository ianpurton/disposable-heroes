"""Build the jeep model in a fresh Blender scene."""
from _common import *

def build(context):
    clear();cube('Jeep body',(0,0,.6),(1.4,2.4,.5),olive);cube('Bonnet',(0,-.7,.95),(1.3,1.,.3),olive)
    for x in [-.75,.75]:
     for y in [-.8,.8]:o=rod('Wheel',(x-.12,y,.4),(x+.12,y,.4),.36,dark)
    for x in [-.5,.5]:cube('Seat',(x,.5,1.),(.4,.6,.45),wood)
