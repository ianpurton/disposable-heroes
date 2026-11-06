---
name: 3d-assets
description: Build procedural Python models into validated GLBs using Blender, with snapshots and contact sheets.
---

# 3D assets

- Keep authoritative Python definitions in `assets/models/` and generated GLBs in
  `assets/glb/`. Use a project Justfile to call [scripts/build.py](scripts/build.py).
- Each public `<name>.py` defines `build(context)` to construct one scene, without
  exporting. Prefix shared helpers with `_`. Declare required clips in `ANIMATIONS`.
- The runner seeds `random` and provides `context.seed`. An optional `_textures.py`
  batch hook declares `OUTPUTS` and writes to `context.texture_directory`.
- Use `--model NAME` for one asset; `--blends DIR` saves packed inspection snapshots.
  Python stays authoritative. `--help` lists overrides; default paths use the
  working directory, not the skill location.
- Use [scripts/preview.py](scripts/preview.py) for a bounds-fitted contact sheet;
  it accepts a GLB directory or manifest, optional ordering and render settings.
- Match the consuming project's scale, pivots, axes and animation names. Inspect
  exports in the target application; keep test previews and logs in `./tmp`.

Use isolated builds, validation and staged publication. Test repeatability and
failure preservation when changing the pipeline.
