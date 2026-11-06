"""Blender integration checks. Run with python3 tests/asset_pipeline.py."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / '.agents/skills/build-blender-assets/scripts/build.py'
spec = importlib.util.spec_from_file_location('asset_builder', RUNNER)
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)
CLIPS = {'Idle', 'Run', 'Fire', 'Throw', 'Rocket', 'Swim', 'Death'}


class AssetPipeline(unittest.TestCase):
    def test_complete_library_and_clean_animation_contract(self):
        manifest = json.loads((ROOT / 'assets/glb/manifest.json').read_text())
        definitions = {p.stem for p in (ROOT / 'assets/models').glob('*.py') if not p.name.startswith('_')}
        self.assertEqual(set(manifest['models']), definitions)
        self.assertEqual(len(definitions), 15)
        for name in definitions:
            with self.subTest(name=name):
                model = ROOT / 'assets/glb' / (name + '.glb')
                required = CLIPS if name in ('soldier', 'enemy') else ()
                actual = builder.inspect_glb(model, required)
                self.assertEqual(actual['semantic_sha256'], manifest['models'][name]['semantic_sha256'])
                self.assertEqual(actual['sha256'], manifest['models'][name]['sha256'])
                self.assertTrue((ROOT / 'assets/models' / manifest['models'][name]['source']).is_file())
                if required:
                    self.assertEqual(set(actual['animations']), CLIPS)
                    self.assertEqual(len(actual['animations']), 7)
                    self.assertEqual(actual['skins'], 1)
        texture = ROOT / 'assets/textures/jungle_ground.png'
        self.assertEqual(hashlib.sha256(texture.read_bytes()).hexdigest(), manifest['textures'][texture.name]['sha256'])

    def test_truncated_glb_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'bad.glb'
            path.write_bytes((ROOT / 'assets/glb/soldier.glb').read_bytes()[:-4])
            with self.assertRaises(ValueError):
                builder.inspect_glb(path)

    def test_batch_failure_preserves_previous_assets_and_manifest(self):
        with tempfile.TemporaryDirectory(prefix='failure with spaces ', dir=ROOT / 'tmp') as temporary:
            directory = Path(temporary)
            source = directory / 'source'; source.mkdir()
            output = directory / 'output'; output.mkdir()
            (source / 'aaa_valid.py').write_text('import bpy\ndef build(context):\n    bpy.ops.mesh.primitive_cube_add()\n')
            (source / 'zzz_broken.py').write_text('def build(context):\n    raise RuntimeError("intentional pipeline failure")\n')
            previous = (ROOT / 'assets/glb/soldier.glb').read_bytes()
            (output / 'aaa_valid.glb').write_bytes(previous)
            old_manifest = b'{"previous": "preserve me"}\n'
            (output / 'manifest.json').write_bytes(old_manifest)
            result = subprocess.run([sys.executable, str(RUNNER), '--source', str(source), '--output', str(output),
                                     '--logs', str(ROOT / 'tmp/asset-pipeline-tests/failure')], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('zzz_broken', result.stderr)
            self.assertEqual((output / 'aaa_valid.glb').read_bytes(), previous)
            self.assertEqual((output / 'manifest.json').read_bytes(), old_manifest)
            self.assertFalse((output / 'zzz_broken.glb').exists())

    def test_independent_soldier_build_works_outside_repository(self):
        with tempfile.TemporaryDirectory(prefix='single with spaces ', dir=ROOT / 'tmp') as temporary:
            directory = Path(temporary)
            before = json.loads((ROOT / 'assets/glb/manifest.json').read_text())['models']['soldier']
            result = subprocess.run([sys.executable, str(RUNNER), '--model', 'soldier', '--output', str(directory / 'glb'),
                                     '--logs', str(ROOT / 'tmp/asset-pipeline-tests/single')], cwd='/tmp', capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            actual = builder.inspect_glb(directory / 'glb/soldier.glb', CLIPS)
            self.assertEqual(actual['semantic_sha256'], before['semantic_sha256'])
            self.assertEqual(list((directory / 'glb').glob('*.glb')), [directory / 'glb/soldier.glb'])

    def test_repeat_build_preserves_all_semantic_hashes(self):
        manifest = json.loads((ROOT / 'assets/glb/manifest.json').read_text())
        with tempfile.TemporaryDirectory(prefix='repeat-', dir=ROOT / 'tmp') as temporary:
            output = Path(temporary) / 'glb'
            result = subprocess.run([sys.executable, str(RUNNER), '--output', str(output), '--textures', str(Path(temporary) / 'textures'),
                                     '--logs', str(ROOT / 'tmp/asset-pipeline-tests/repeat')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            repeated = json.loads((output / 'manifest.json').read_text())
            for name, record in manifest['models'].items():
                self.assertEqual(record['semantic_sha256'], repeated['models'][name]['semantic_sha256'], name)
            self.assertEqual(manifest['textures'], repeated['textures'])


if __name__ == '__main__':
    (ROOT / 'tmp').mkdir(exist_ok=True)
    unittest.main(verbosity=2)
