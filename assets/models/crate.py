"""Build the crate model in a fresh Blender scene."""
from _common import *

def build(context):
    clear();cube('Ammo crate',(0,0,.25),(.6,.6,.5),olive);cube('Marking',(0,-.31,.28),(.35,.02,.12),roof);
