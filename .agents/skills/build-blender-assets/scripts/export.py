"""Blender-side worker. Invoked by build.py, one fresh process per model."""
import argparse
import json
from pathlib import Path
import random
import runpy
import sys
from types import SimpleNamespace

import bpy


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--stage', type=Path, required=True)
    parser.add_argument('--model', required=True)
    parser.add_argument('--seed', type=int, default=1993)
    parser.add_argument('--save-blend', action='store_true')
    args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    random.seed(args.seed)
    sys.path.insert(0, str(args.source))
    module = runpy.run_path(str(args.source / (args.model + '.py')))
    context = SimpleNamespace(seed=args.seed, texture_directory=args.stage / 'textures')
    module['build'](context)
    info = {'blender_version': bpy.app.version_string, 'seed': args.seed}
    if args.model == '_textures':
        outputs = list(module['OUTPUTS'])
        for name in outputs:
            if Path(name).name != name or not (context.texture_directory / name).is_file():
                raise ValueError(f'Texture hook did not produce declared output: {name}')
        info['outputs'] = outputs
    else:
        if not any(obj.type == 'MESH' for obj in bpy.context.scene.objects):
            raise ValueError(f'{args.model}: scene contains no mesh')
        expected = list(module.get('ANIMATIONS', ()))
        info['required_animations'] = expected
        bpy.ops.export_scene.gltf(
            filepath=str(args.stage / (args.model + '.glb')),
            export_format='GLB', export_yup=True,
            export_animations=bool(expected), export_animation_mode='ACTIONS',
            export_force_sampling=True,
        )
    if args.save_blend:
        # Generated texture hooks may have no material users; retain their pixels.
        for image in bpy.data.images:
            if image.has_data:
                image.use_fake_user = True
        bpy.ops.file.pack_all()
        bpy.ops.wm.save_as_mainfile(
            filepath=str(args.stage / (args.model + '.blend')),
            check_existing=False, compress=True,
        )
    (args.stage / (args.model + '.info.json')).write_text(json.dumps(info))


if __name__ == '__main__':
    main()
