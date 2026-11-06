---
name: build-blender-assets
description: Create or modify procedural Python model definitions and build validated GLBs with Blender. Use for this repository's script-driven model pipeline, single-model rebuilds, or inspection snapshots; not for hand-authored Blender files or full level authoring.
---

# Build Blender assets

Python model definitions are authoritative. GLBs and optional `.blend` snapshots
are generated outputs; manual edits to those files are overwritten.

## Author and build

- Definitions live in `assets/models/`; each public `<name>.py` exports
  `build(context)`. It constructs one model in the current scene and does not
  launch Blender, save files, or export. Modules beginning with `_` are helpers.
- `context.seed` is the build seed (1993 by default). The worker seeds Python's
  `random` module before importing the definition. Preserve explicit offsets when
  extracting variants from a previously shared random sequence.
- Models with a rig declare `ANIMATIONS`, the exact required clip names. The worker
  rejects missing/extra clips or skins. Static models omit it.
- The optional `_textures.py` hook runs once for an all-model build. Its
  `build(context)` writes only declared `OUTPUTS` into `context.texture_directory`.
  A single-model build does not regenerate shared textures.
- Run `just glbs` for the library or `just glb soldier` for one model. Use
  `just blends` to also save packed inspection snapshots under `tmp/blender/`.
- The runner is [scripts/build.py](scripts/build.py); it can be called directly
  with `--source`, `--output`, `--textures`, `--logs`, `--model`, `--blends`,
  `--seed`, `--threads`, and `--blender`. Defaults resolve from the repository,
  independently of the working directory. `BLENDER` overrides the executable.

## Asset conventions

- Work in meters, Blender Z-up. glTF export converts to Bevy Y-up. Existing soldier
  definitions face Blender -Y, which becomes Bevy +Z; retain that convention.
- Ground props use a ground-centred pivot. Record intentional offsets and footprints
  in the definition. Do not indiscriminately apply transforms on a rigged asset.
- Keep names used by animation masks and attachments stable. Each model gets a
  fresh Blender process, so actions and materials cannot leak from another model.
- GLBs embed texture dependencies. Packed `.blend` snapshots preserve inspection
  data, including named actions; they are not an alternative source of truth.

## Validate

The runner uses Blender's Python exception exit code, validates the GLB container,
embedded buffers, scene/mesh presence, and required animation/skin contract, and
stages outputs before publishing. A failed batch does not replace prior assets or
its manifest. Logs remain in `tmp/blender-build/`; inspect the failing model log.
The manifest in `assets/glb/` records source paths, Blender version, seed, counts,
and file/semantic hashes. Semantic hashes round decoded floats to four decimal
places and exclude unused UV values in untextured models. File hashes retain
exact-byte integrity, including UV data. Single builds merge their record into the manifest.

After a model change, inspect its GLB in Bevy. For pipeline changes, build the whole
library, independently rebuild a soldier, compare semantic hashes across two builds,
and exercise failure preservation. Use `scripts/preview_assets.py` for a contact
sheet and the game's `--smoke-test` / `--combat-preview` for runtime captures.
All test images and logs belong in `./tmp`. Do not install Blender into the game
runtime or modify maps as part of an asset export.
