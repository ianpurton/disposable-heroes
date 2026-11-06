# Implementation status

This is the initial playable foundation, not a finished fidelity-validated release.

Implemented: Rust-only runtime, native CF1 MAP/SPT/HIT/BHT/SWP import, real pinned fixtures, inspector, three imported jungle phases, mouse selection/movement/shooting, split/regroup, grenades/rockets and pickups, enemy pursuit/fire, barracks reinforcements, destruction and terrain substitutions, mission outcomes, mission-start retry, named roster persistence and memorial, generated GLB library with animation clips, terrain, water, lighting, shadows, effects, synthesized audio, HUD and screenshot capture.

Remaining before the approved polished slice is complete:

- Compare movement, timing, ranges, enemy behavior and control gestures against Amiga reference footage. Current tuning is provisional.
- Expand structure recognition beyond the jungle barracks pattern, validate sprite-specific anchors, and handle bridges and bunker variants accurately in 3D.
- Author the final showcase mission using the existing editor. The current third mission uses AmigaAction/mapm3 unchanged.
- Fully support civilian behaviors and campaign aggression ranges. Unsupported objectives are rejected; some decorative sprite IDs currently have no 3D representation.
- Improve soldier animation transitions, upper-body masks, explosive poses, swimming integration and foliage occlusion handling.
- Improve painted surface detail, destruction debris, lighting and sound mixing.
- Validate performance on the Linux host. The initial Xvfb screenshot used software Vulkan; smoke-test capture now renders offscreen to avoid Xvfb hardware presentation requirements.

Validation: 12 core tests pass; Linux build and all-target checks pass; offscreen Vulkan smoke capture passed on AMD Radeon Graphics (RADV GFX1150). Preview images are in `tmp/`.

No Windows build was run locally. CI configuration is provided, but its remote results have not been observed.
