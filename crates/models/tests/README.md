The legacy fixtures record measurements from the seed-1993 Blender exports before
removing the old pipeline. `legacy-rig.json` contains local rest transforms and
exported animation channel samples; `legacy-models.json` contains world bounds
and linear material palettes. They are regression references, not runtime input.
No geometry is baked into the Rust generator. The detailed recipes retain the
original anchors and approximate footprints: actors and small props allow 8 cm
aggregate bounds variation, buildings and bridges 30 cm, and larger scenery
50 cm for roots, framing and equipment. Jungle silhouettes have been replaced.
Rig channels and inverse bind frames retain the strict legacy tolerances.
Surface tests cover finite projected UVs and normalized procedural ground normals.

Before removing the legacy files, compare Bevy-rendered contact sheets and game
captures under `tmp/`. The optional export test uses the independent glTF reader
to validate every exported model, including complete skins, animations, texture coordinates and embedded material textures.
