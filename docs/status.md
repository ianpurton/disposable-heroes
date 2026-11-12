# Implementation status

The first-mission polish pass is implemented; host acceptance and Amiga timing comparisons remain pending.

Implemented: native CF1 import and pinned reference tile decoding; reviewed visual mappings for every first-mission tile; Rust-authoritative procedural models and sound in crates/models and crates/sound, consumed directly by crates/game, with a Bevy gallery and optional GLB export; generated, merged jungle thickets and baked ground detail replacing collision slabs; deterministic two-pixel navigation with clearance, corner checks and reachable-goal handling; leader trails, independent split squads and leader-death continuation; destruction rebuilding routes; visibility-based enemy reaction/search states and campaign aggression; aim-aligned muzzle flashes, impacts, bounded particles, brief explosion lighting, order/pickup feedback; shared animation graphs, transitions, upper-body masks, swimming/throw/rocket/death poses; foliage lowering near selected soldiers; repeatable 1080p release benchmark and native input smoke script.

The existing three imported jungle phases, roster persistence, mission retry, memorial, camera tracking, HUD, generated GLB library and audio remain available. The first mission is the reviewed environment target; the other phases need the same tile-by-tile visual review.

Prior baseline validation: 25 tests pass (12 core, 11 navigation/environment/combat, 2 camera/facing); Linux dev/release builds and all-target compile checks pass. AMD Vulkan overview and scripted combat captures are in tmp/. Performance results are recorded separately in tmp/benchmark.json.

Reference graphics refresh: generated surface textures and projected UVs shared by
the game/gallery/GLB exporter; soil albedo with stones and leaf litter plus normal
relief; folded palm leaflets, layered broadleaf crowns, roots and mixed forest
heights; detailed building frames, props, vehicles and soldier equipment; native
HDR, bloom, ambient occlusion and FXAA; bounded textured flames, smoke, sparks,
debris and fading scorch marks. Material/bone merging, solid canopy shadow
casters and a single orthographic shadow cascade keep rendering costs bounded.
Map collision, objectives, unit dimensions, rigs and animation samples remain
compatible. Browser builds use smaller shadows and fewer particles without
native postprocessing.

A 20-second 1080p release benchmark on AMD Radeon Graphics (RADV PHOENIX)
measured p50/p95/p99 of 14.07/15.08/15.58 ms over 1,414 samples, after three
seconds of warmup. This is the scripted first mission offscreen on Vulkan, with
no compositor or vsync; it is not a host interaction or all-mission guarantee.
Captures and reports are under `tmp/`. The refresh passes all 71 workspace tests
with export support, all-target native checking and the wasm32 library/game
compatibility check. Rest, running and firing gallery captures were inspected,
along with fresh battlefield and combat captures.

Remaining work:

- Host playtest navigation, combat difficulty, readability and audio mix. The container has no audio output device.
- Compare travel times, follow spacing, reaction, fire cadence and terrain rules against an Amiga playthrough. Current tuning and visibility interpretation are explicitly provisional; see docs/first-mission.md.
- Review additional mission tile mappings, sprite anchors, bridge/bunker families and civilian behavior.
- Author the final showcase mission in the existing editor; the third phase still uses AmigaAction/mapm3 unchanged.
- Assess measured release performance on the user's host before claiming the 60 fps target.

No Windows build was run locally. CI configuration is provided, but remote results have not been observed.
