---
name: 3d-assets
description: Author procedural 3D assets as Python definitions and build validated GLBs through Blender, with optional inspection snapshots.
---

# 3D assets

- Keep authoritative Python definitions in `assets/models/` and generated GLBs in
  `assets/glb/`. Use a project Justfile to call [scripts/build.py](scripts/build.py).
- Each public `<name>.py` defines `build(context)` to construct one scene, without
  exporting. Prefix shared helpers with `_`. Declare required clips in `ANIMATIONS`.
- The runner seeds `random` and provides `context.seed`. An optional `_textures.py`
  batch hook declares `OUTPUTS` and writes to `context.texture_directory`.
- Use `--model NAME` for one asset; `--blends DIR` saves packed inspection snapshots.
  Python stays authoritative. Run `--help` for path and Blender overrides; defaults
  resolve from the working directory, not the skill's installation location.
- Match the consuming project's scale, pivots, axes and animation names. Inspect
  exports in the target application; keep test previews and logs in `./tmp`.

Builds isolate models in fresh Blender processes, validate exports, and stage files
before publication. Failures preserve prior outputs. Check the export manifest and
per-model logs; test repeatability and failure preservation after pipeline changes.
