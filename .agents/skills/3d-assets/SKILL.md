---
name: 3d-assets
description: Author, preview, validate and optionally export deterministic procedural Bevy models and sounds in Rust, or assemble exported GLBs into a Blender inspection file.
---

# Procedural assets

- `crates/models` is authoritative for geometry, materials, rigs, clips and the
  procedural ground texture. `crates/sound` is authoritative for synthesized PCM.
  `crates/game` consumes generated assets directly. Never introduce GLB, Blender,
  Python or generated files into the normal runtime or build dependency chain.
- Model recipes live in `crates/models/src/library/<model>.rs`; shared builder
  helpers and dispatch live in `library.rs`, and shared geometry in `geometry.rs`. Add public model names to `models::NAMES`. Generate with
  `models::generate(name, GenerationOptions { seed })`, validate, then prepare
  handles once with `PreparedModel::new` and instantiate with `spawn`.
- Coordinates in recipes retain the original X-right, Z-up, -Y-forward convention;
  the builder converts once to Bevy Y-up. Preserve dimensions, pivots, linear
  material colors and flat shading. Do not convert model colors to sRGB twice.
- Preserve `SoldierRig`, the six named bones and the seven `models::CLIPS` names.
  Rig pose channels are bone-local; retain rest orientations, inverse binds,
  frame times and upper/lower body masks. Use the same pose samples for Bevy clips
  and optional export. Do not create a second animation implementation.
- Generation must be deterministic and independent of catalog iteration and
  gameplay RNG. Default seed is 1993; the game exposes `--asset-seed`. The small
  shared internal MT19937 helper in `crates/seed.rs` preserves legacy sequences.
  Thicket edge skips 60 draws; ground skips 120; sounds share a sequential stream.
- `just assets-build` compiles the gallery. `just assets-preview [model] [seed]`
  opens it. `just assets-capture [output] [seed] [clip] [time]` writes a labelled,
  bounds-fitted contact sheet without a desktop display. Omit clip for rest pose.
  Direct gallery CLI supports `--model`, `--seed`, `--clip`, `--time`, `--output`.
  Clip without time plays continuously. Keep all inspection images under `tmp/`.
- Inspect the gallery image and representative animated poses after visual edits.
  Lighting, bounds fitting, generation and rigging must match game consumption.
  Use real Vulkan hardware when available; interactive playtesting is on the host.
- Run `just assets-validate` for workspace tests and WASM checks. Tests cover
  generation, geometry, bind frames, legacy clip samples, audio and GLB roundtrip.
  Also run the game's `--fresh --smoke-test` for integration changes.
- `just assets-export [model] [seed] [output-directory]` enables the optional native
  exporter and writes GLBs under `tmp/glb` by default. Export is for interoperability,
  never an intermediate step. The exporter consumes the same generated model data
  and publishes each file by temporary-file replacement. Inspect exported animated
  characters in a consuming application when changing export behavior.
- Keep geometry/sound algorithms and recipe parameters readable Rust. Add narrow
  helpers for repeated construction; avoid generic authoring frameworks or baked
  vertex dumps. Tests should check contracts and behavior, not mirror recipes.

## Blender inspection export

Use the bundled [scripts/blender_assets.py](scripts/blender_assets.py) when the
user wants a `.blend` file for inspecting exported models. It is independent of
the game, Rust and Just: it reads any directory of `.glb` files using Blender's
bundled Python. Requires Blender with its glTF importer (verified with 5.2.3).

Resolve the script path relative to this skill's location. From any project or
working directory, run (replace the example paths):

```sh
blender --background --factory-startup --python-exit-code 1 \
  --python /path/to/3d-assets/scripts/blender_assets.py -- \
  --input /path/to/glbs --output /path/to/asset-library.blend
```

The helper imports every top-level `.glb`, creates a named collection per file,
spaces models in a grid at their original sizes, retains imported materials,
rigs and animation clips, and saves a framed inspection viewport. It creates a
new scene rather than editing an existing `.blend`; the output path is replaced
if it already exists. Empty inputs or failed imports exit nonzero. Reopen the
saved file to check the expected model collections and any rigs/clips.

In this repository, `just assets-blend [output] [seed]` exports the Rust library
into a temporary GLB directory, calls this helper, then cleans up the temporary
files. Defaults are `tmp/asset-library.blend` and seed `1993`. In another project,
use that project's exporter or existing GLBs and call the helper directly; the
repository-specific Rust layout and recipes above are not helper dependencies.
Keep Blender export optional and outside normal game builds.
