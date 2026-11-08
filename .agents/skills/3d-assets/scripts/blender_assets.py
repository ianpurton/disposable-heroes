"""Assemble a directory of GLBs into a Blender inspection scene.

Run with Blender's bundled Python, not system Python. Inputs and output are
explicit paths; this helper has no dependency on a Rust workspace or Justfile.
"""

import argparse
import math
from pathlib import Path
import sys

import bpy
from mathutils import Vector


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1 :])
    files = sorted(args.input.glob("*.glb"))
    if not files:
        raise ValueError(f"No GLBs found in {args.input}")
    if args.output.suffix != ".blend":
        raise ValueError("Output must have a .blend extension")

    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    assets = []
    for path in files:
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(path.resolve()))
        objects = set(bpy.data.objects) - before
        collection = bpy.data.collections.new(path.stem)
        scene.collection.children.link(collection)
        root = bpy.data.objects.new(f"{path.stem} layout", None)
        collection.objects.link(root)
        for obj in objects:
            for old_collection in list(obj.users_collection):
                old_collection.objects.unlink(obj)
            collection.objects.link(obj)
            if obj.parent is None:
                obj.parent = root
        # Move only the new parent: imported mesh/rig transforms and clips stay intact.
        scene.frame_set(0)
        bpy.context.view_layer.update()
        corners = [
            obj.matrix_world @ Vector(corner)
            for obj in objects if obj.type == "MESH"
            for corner in obj.bound_box
        ]
        if not corners:
            raise ValueError(f"No mesh geometry in {path}")
        lo = Vector(tuple(min(p[i] for p in corners) for i in range(3)))
        hi = Vector(tuple(max(p[i] for p in corners) for i in range(3)))
        assets.append((root, lo, hi))

    # Preserve real model sizes, with enough space for the largest footprint.
    columns = min(5, len(assets))
    rows = math.ceil(len(assets) / columns)
    spacing = max(max((hi - lo).x, (hi - lo).y) for _, lo, hi in assets) * 1.3
    for index, (root, lo, hi) in enumerate(assets):
        root.location = (
            (index % columns - (columns - 1) / 2) * spacing - (lo.x + hi.x) / 2,
            ((rows - 1) / 2 - index // columns) * spacing - (lo.y + hi.y) / 2,
            -lo.z,
        )

    # Open straight into a framed, material-colored inspection view.
    rotation = Vector((0, -1.4, 1.8)).to_track_quat("Z", "Y")
    extent = max(columns, rows) * spacing
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type == "VIEW_3D":
                space = area.spaces.active
                space.shading.type = "SOLID"
                space.shading.color_type = "MATERIAL"
                space.clip_end = max(1000, extent * 10)
                space.region_3d.view_rotation = rotation
                space.region_3d.view_location = (0, 0, 0)
                space.region_3d.view_distance = extent * 1.3
                space.region_3d.view_perspective = "ORTHO"
    bpy.ops.object.select_all(action="DESELECT")
    bpy.context.view_layer.objects.active = None
    scene.frame_end = max(1, math.ceil(max(
        (action.frame_range[1] for action in bpy.data.actions), default=1
    )))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(args.output.resolve()))
    print(f"Saved {len(assets)} assets to {args.output}")


if __name__ == "__main__":
    main()
