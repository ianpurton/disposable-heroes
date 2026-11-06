#!/usr/bin/env python3
"""Build procedural Python definitions using isolated Blender workers (stdlib only)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys
import tempfile

def inspect_glb(path, required=()):
    """Check the binary container, embedded resources, meshes, and animation contract."""
    data = path.read_bytes()
    if len(data) < 20 or struct.unpack_from('<4sII', data) != (b'glTF', 2, len(data)):
        raise ValueError(f'{path.name}: invalid GLB header')
    offset = 12
    chunks = []
    while offset < len(data):
        if offset + 8 > len(data):
            raise ValueError('Truncated chunk header')
        size, kind = struct.unpack_from('<II', data, offset)
        offset += 8
        if size % 4 or offset + size > len(data):
            raise ValueError('Truncated or unaligned chunk')
        chunks.append((kind, data[offset:offset + size]))
        offset += size
    if not chunks or chunks[0][0] != 0x4e4f534a:
        raise ValueError('Missing JSON chunk')
    document = json.loads(chunks[0][1])
    binary = b''.join(value for kind, value in chunks if kind == 0x004e4942)
    if document.get('asset', {}).get('version') != '2.0':
        raise ValueError('Expected glTF 2.0')
    if not document.get('scenes') or not document.get('meshes'):
        raise ValueError('Export has no scene or meshes')
    for buffer in document.get('buffers', []):
        if 'uri' in buffer or buffer['byteLength'] > len(binary):
            raise ValueError('Missing embedded GLB buffer')
    for view in document.get('bufferViews', []):
        if view.get('byteOffset', 0) + view['byteLength'] > len(binary):
            raise ValueError('Buffer view exceeds binary data')
    for image in document.get('images', []):
        if 'uri' in image:
            raise ValueError('GLB texture must be embedded')
    names = [animation.get('name') for animation in document.get('animations', [])]
    if len(names) != len(set(names)):
        raise ValueError('Duplicate animation names')
    if required and set(names) != set(required):
        raise ValueError(f'Expected clips {list(required)}; got {names}')
    primitives = [p for mesh in document['meshes'] for p in mesh['primitives']]
    accessors = document.get('accessors', [])
    vertices = sum(accessors[p['attributes']['POSITION']]['count'] for p in primitives)
    if vertices == 0:
        raise ValueError('Export has no vertices')
    # Blender bevel UVs can vary in the last float32 bits across processes.
    # Compare decoded attributes at 0.0001 precision, not incidental buffer bytes.
    def normalized(value):
        if isinstance(value,float):
            value=round(value,4)
            return 0.0 if value==0 else value
        if isinstance(value,(list,tuple)):return [normalized(v) for v in value]
        if isinstance(value,dict):return {k:normalized(v) for k,v in value.items()}
        return value
    semantic = dict(document)
    semantic['asset'] = {'version': document['asset']['version']}
    decoded=[]
    unused_uvs={index for primitive in primitives for name,index in primitive["attributes"].items() if name.startswith("TEXCOORD_")} if not document.get("textures") else set()
    formats={5120:'b',5121:'B',5122:'h',5123:'H',5125:'I',5126:'f'}
    sizes={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT2':4,'MAT3':9,'MAT4':16}
    for accessor in accessors:
        if 'sparse' in accessor or 'bufferView' not in accessor:
            raise ValueError('Sparse or missing accessor storage is not supported by this pipeline')
        view=document['bufferViews'][accessor['bufferView']]
        fmt='<'+formats[accessor['componentType']]*sizes[accessor['type']]
        size=struct.calcsize(fmt);stride=view.get('byteStride',size)
        offset=view.get('byteOffset',0)+accessor.get('byteOffset',0)
        limit=view.get('byteOffset',0)+view['byteLength']
        if stride<size or offset+(accessor['count']-1)*stride+size>limit:
            raise ValueError('Accessor exceeds its buffer view')
        decoded.append("unused UVs in untextured model" if len(decoded) in unused_uvs else [struct.unpack_from(fmt,binary,offset+i*stride) for i in range(accessor['count'])])
    images=[hashlib.sha256(binary[document['bufferViews'][image['bufferView']].get('byteOffset',0):document['bufferViews'][image['bufferView']].get('byteOffset',0)+document['bufferViews'][image['bufferView']]['byteLength']]).hexdigest() for image in document.get('images',[]) if 'bufferView' in image]
    semantic_bytes=json.dumps(normalized([semantic,decoded,images]),sort_keys=True,separators=(',',':')).encode()
    return {
        'sha256': hashlib.sha256(data).hexdigest(),
        'semantic_sha256': hashlib.sha256(semantic_bytes).hexdigest(),
        'semantic_float_precision':4,
        'meshes': len(document['meshes']), 'primitives': len(primitives),
        'vertices': vertices, 'animations': names, 'skins': len(document.get('skins', [])),
    }


def run(args):
    source = args.source.resolve()
    output = args.output.resolve()
    textures = args.textures.resolve()
    logs = args.logs.resolve()
    snapshots = args.blends.resolve() if args.blends else None
    definitions = sorted(p.stem for p in source.glob('*.py') if not p.name.startswith('_'))
    if not definitions:
        raise ValueError(f'No model definitions in {source}')
    names = [args.model] if args.model else definitions
    if any(not re.fullmatch(r'[a-z][a-z0-9_]*', name) or name not in definitions for name in names):
        raise ValueError(f'Unknown model {args.model!r}; available: {", ".join(definitions)}')
    if output == source or output.is_relative_to(source):
        raise ValueError('Generated output must be separate from model sources')
    output.parent.mkdir(parents=True, exist_ok=True)
    logs.mkdir(parents=True, exist_ok=True)
    # Staging is on the output filesystem, allowing atomic per-file replacement.
    with tempfile.TemporaryDirectory(prefix='.glb-build-', dir=output.parent) as temporary:
        stage = Path(temporary)
        jobs = names + (['_textures'] if not args.model and (source / '_textures.py').is_file() else [])
        models = {}
        texture_records = {}
        version = None
        for name in jobs:
            command = [args.blender, '--background', '--factory-startup', '--threads', str(args.threads),
                       '--python-exit-code', '1', '--python', str(Path(__file__).with_name('export.py')),
                       '--', '--source', str(source), '--stage', str(stage), '--model', name,
                       '--seed', str(args.seed)]
            if snapshots:
                command.append('--save-blend')
            log = logs / (name + '.log')
            print(f'Building {name} ...', flush=True)
            with log.open('w') as handle:
                result = subprocess.run(command, stdout=handle, stderr=subprocess.STDOUT)
            if result.returncode:
                raise RuntimeError(f'{name}: Blender exited {result.returncode}; see {log}')
            info = json.loads((stage / (name + '.info.json')).read_text())
            version = info['blender_version']
            if name == '_textures':
                for filename in info['outputs']:
                    texture_records[filename] = {'sha256': hashlib.sha256((stage / 'textures' / filename).read_bytes()).hexdigest(), 'source': '_textures.py'}
            else:
                models[name] = dict(inspect_glb(stage / (name + '.glb'), info['required_animations']),
                                    source=name + '.py', output=name + '.glb',
                                    blender_version=version, seed=args.seed)
        # Nothing is published until every requested worker and GLB validation succeeds.
        output.mkdir(parents=True, exist_ok=True)
        manifest_path = output / 'manifest.json'
        previous = json.loads(manifest_path.read_text()) if args.model and manifest_path.exists() else {}
        entries = previous.get('models', {})
        entries.update(models)
        manifest = {
            'schema_version': 1, 'generator': '3d-assets', 'blender_version': version,
            'seed': args.seed, 'units': 'meters; Blender Z-up converted to glTF Y-up',
            'source_directory': os.path.relpath(source, output).replace(os.sep, '/'),
            'models': dict(sorted(entries.items())),
            'assets': sorted(entry['output'] for entry in entries.values()),
            'textures': texture_records if not args.model else previous.get('textures', {}),
        }
        # Only declared files are replaced. Unrelated files are never removed.
        for name in names:
            os.replace(stage / (name + '.glb'), output / (name + '.glb'))
        if texture_records:
            textures.mkdir(parents=True, exist_ok=True)
            for filename in texture_records:
                os.replace(stage / 'textures' / filename, textures / filename)
        if snapshots:
            snapshots.mkdir(parents=True, exist_ok=True)
            for name in jobs:
                os.replace(stage / (name + '.blend'), snapshots / (name + '.blend'))
        manifest_stage = stage / 'manifest.json'
        manifest_stage.write_text(json.dumps(manifest, indent=2) + '\n')
        os.replace(manifest_stage, manifest_path)
    print(f'Built {len(names)} GLBs in {output}; Blender {version}', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('assets/models'))
    parser.add_argument('--output', type=Path, default=Path('assets/glb'))
    parser.add_argument('--textures', type=Path, default=Path('assets/textures'))
    parser.add_argument('--logs', type=Path, default=Path('tmp/blender-build'))
    parser.add_argument('--blends', type=Path, help='Optional generated .blend snapshots')
    parser.add_argument('--model', help='Build one named model; omit for the entire library')
    parser.add_argument('--blender', default=os.environ.get('BLENDER', 'blender'))
    parser.add_argument('--threads', type=int, default=2)
    parser.add_argument('--seed', type=int, default=0)
    args = parser.parse_args()
    if args.threads < 1:
        parser.error('--threads must be positive')
    try:
        run(args)
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print(f'Asset build failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
