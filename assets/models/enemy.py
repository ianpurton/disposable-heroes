"""Build the enemy model. Python is authoritative; GLBs are generated."""
from _common import enemy
from _soldier import soldier

ANIMATIONS = ("Idle", "Run", "Fire", "Throw", "Rocket", "Swim", "Death")

def build(context):
    soldier("enemy", enemy)
