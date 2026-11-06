# Build the complete procedural model library.
default: glbs

glbs:
    python3 .agents/skills/build-blender-assets/scripts/build.py

# Rebuild one named model, for example: just glb soldier.
glb name:
    python3 .agents/skills/build-blender-assets/scripts/build.py --model {{quote(name)}}

# Save generated inspection snapshots; Python definitions remain authoritative.
blends:
    python3 .agents/skills/build-blender-assets/scripts/build.py --blends tmp/blender
