"""Build the soldier model. Python is authoritative; GLBs are generated."""
from _common import olive
from _soldier import soldier

ANIMATIONS = ("Idle", "Run", "Fire", "Throw", "Rocket", "Swim", "Death")

def build(context):
    soldier("soldier", olive)
