"""Build thicket with a one-meter footprint and ground-centred pivot."""
from _jungle import build_thicket
import random

def build(context):
    # The original family generator consumed 60 random values per thicket.
    for _ in range(0):
        random.random()
    build_thicket("thicket")
