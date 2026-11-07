The legacy fixtures record measurements from the seed-1993 Blender exports before
removing the old pipeline. `legacy-rig.json` contains local rest transforms and
exported animation channel samples; `legacy-models.json` contains world bounds
and linear material palettes. They are regression references, not runtime input.
No geometry is baked into the Rust generator. Bounds allow 3.5 cm aggregate
variation for independently tessellated bevels and primitive orientation rounding;
rig channels and material comparisons use much tighter tolerances.

Before removing the legacy files, compare Bevy-rendered contact sheets and game
captures under `tmp/`. The optional export test uses the independent glTF reader
to validate every exported model, including complete skins and animations.
