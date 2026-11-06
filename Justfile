# Build the complete procedural model library.
default: glbs

glbs:
    python3 .agents/skills/3d-assets/scripts/build.py --seed 1993

# Rebuild one named model, for example: just glb soldier.
glb name:
    python3 .agents/skills/3d-assets/scripts/build.py --seed 1993 --model {{quote(name)}}

# Save generated inspection snapshots; Python definitions remain authoritative.
blends:
    python3 .agents/skills/3d-assets/scripts/build.py --seed 1993 --blends tmp/blender
