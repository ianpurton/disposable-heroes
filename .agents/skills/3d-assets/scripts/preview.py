#!/usr/bin/env python3
"""Render an automatically framed GLB contact sheet using Blender (stdlib launcher)."""
import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import sys


def render(args, paths):
    import bpy
    from mathutils import Vector
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    columns = min(args.columns, len(paths))
    rows = math.ceil(len(paths) / columns)
    spacing = 4.5
    for index, path in enumerate(paths):
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(path))
        imported = set(bpy.data.objects) - before
        scene.frame_set(1)
        bpy.context.view_layer.update()
        depsgraph = bpy.context.evaluated_depsgraph_get()
        corners = [obj.matrix_world @ Vector(corner) for obj in imported if obj.type == 'MESH'
                   for corner in obj.evaluated_get(depsgraph).bound_box]
        if not corners:
            raise ValueError(f'No mesh bounds in {path}')
        low = Vector(tuple(min(p[axis] for p in corners) for axis in range(3)))
        high = Vector(tuple(max(p[axis] for p in corners) for axis in range(3)))
        scale = 3.2 / max(*(high - low), 0.001)
        wrapper = bpy.data.objects.new(path.stem + ' preview', None)
        scene.collection.objects.link(wrapper)
        for obj in imported:
            if obj.parent not in imported:
                obj.parent = wrapper
        wrapper.scale = (scale,) * 3
        x, y = index % columns * spacing, index // columns * spacing
        wrapper.location = (x - (low.x + high.x) * 0.5 * scale,
                            y - (low.y + high.y) * 0.5 * scale, -low.z * scale)
        bpy.ops.object.text_add(location=(x - 1.65, y - 1.95, 0.02))
        label = bpy.context.object
        label.data.body = path.stem
        label.data.size = 0.28
    bpy.ops.mesh.primitive_plane_add(size=max(columns, rows) * spacing * 5)
    bpy.context.object.location.z = -0.04
    material = bpy.data.materials.new('Preview ground')
    material.diffuse_color = (0.12, 0.15, 0.13, 1)
    bpy.context.object.data.materials.append(material)
    center = Vector(((columns - 1) * spacing / 2, (rows - 1) * spacing / 2, 1.2))
    bpy.ops.object.camera_add(location=center + Vector((16, -24, 28)))
    camera = bpy.context.object
    camera.rotation_euler = (center - camera.location).to_track_quat('-Z', 'Y').to_euler()
    camera.data.type = 'ORTHO'
    # Fit the entire grid in camera space, including labels and tall models.
    corners = [Vector((x, y, z)) for x in (-2.3, (columns - 1) * spacing + 2.3)
               for y in (-2.3, (rows - 1) * spacing + 2.3) for z in (0, 3.3)]
    inverse = camera.rotation_euler.to_matrix().transposed()
    projected = [inverse @ (p - center) for p in corners]
    width = max(p.x for p in projected) - min(p.x for p in projected)
    height = max(p.y for p in projected) - min(p.y for p in projected)
    aspect = args.width / args.height
    # Orthographic scale is the camera width; convert height using render aspect.
    camera.data.ortho_scale = max(width, height * aspect) * 1.12
    scene.camera = camera
    bpy.ops.object.light_add(type='AREA', location=center + Vector((-3, -6, 14)))
    light = bpy.context.object
    light.rotation_euler = (center - light.location).to_track_quat('-Z', 'Y').to_euler()
    light.data.energy = 2600
    light.data.shape = 'DISK'
    light.data.size = 12
    scene.world = bpy.data.worlds.new('Preview world')
    scene.world.color = (0.3, 0.3, 0.3)
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = args.samples
    scene.render.resolution_x, scene.render.resolution_y = args.width, args.height
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'
    args.output.parent.mkdir(parents=True, exist_ok=True)
    scene.render.filepath = str(args.output)
    bpy.ops.render.render(write_still=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, default=Path('assets/glb'))
    parser.add_argument('--manifest', type=Path, help='Manifest with an assets list; paths are relative to its directory')
    parser.add_argument('--output', type=Path, default=Path('tmp/asset-library.png'))
    parser.add_argument('--order', default='', help='Optional comma-separated model names to show first')
    parser.add_argument('--columns', type=int, default=4)
    parser.add_argument('--width', type=int, default=1400)
    parser.add_argument('--height', type=int, default=1000)
    parser.add_argument('--samples', type=int, default=16)
    parser.add_argument('--threads', type=int, default=4)
    parser.add_argument('--blender', default=os.environ.get('BLENDER', 'blender'))
    parser.add_argument('--blender-worker', action='store_true', help=argparse.SUPPRESS)
    argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)
    if min(args.columns, args.width, args.height, args.samples, args.threads) < 1:
        parser.error('columns, dimensions, samples and threads must be positive')
    try:
        paths = [args.manifest.parent / name for name in json.loads(args.manifest.read_text())['assets']] if args.manifest else list(args.input.glob('*.glb'))
        preferred = args.order.split(',') if args.order else []
        paths = sorted((p.resolve() for p in paths), key=lambda p: (preferred.index(p.stem) if p.stem in preferred else len(preferred), p.name))
        if not paths or any(not p.is_file() for p in paths):
            raise ValueError('No GLBs found, or manifest references missing files')
        args.output = args.output.resolve()
        if args.blender_worker:
            render(args, paths)
            return 0
        return subprocess.run([args.blender, '--background', '--factory-startup', '--threads', str(args.threads),
                               '--python-exit-code', '1', '--python', str(Path(__file__).resolve()),
                               '--', '--blender-worker', *argv]).returncode
    except (OSError, ValueError, KeyError) as error:
        print(f'Preview failed: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
