# Disposable Heroes

A Rust / Bevy 0.20 jungle squad-combat project inspired by Cannon Fodder. Native OpenFodder map import, Rust-generated models and audio, fixed-step combat, named soldiers and mission retry.

## Linux host

Run from the repository root. The binary targets `x86_64-unknown-linux-gnu`. Use `just build` (or plain `just`) to rebuild the game, and `just play` to build and launch it with your saved campaign. `just play --fresh` starts a new campaign. Asset gallery commands build the gallery separately.

```sh
python3 scripts/fetch_fixtures.py
just play --fresh
```

The workspace contains `crates/game` (the `disposable-heroes` package),
`crates/models` and `crates/sound`. Run commands from the repository root; the
workspace defaults to the game package. Models, materials, rigs, animations,
ground texture and audio are generated directly in memory. Building and running
requires neither Python nor Blender nor generated GLBs/WAVs. Python is used only
for the separate original-map fixture download and reference tools.

```sh
just assets-build                         # compile native gallery
just assets-preview                       # labelled library; optional model and seed
just assets-capture                       # offscreen tmp/asset-library.png
just assets-capture tmp/run.png 1993 Run 0.5
just assets-export soldier                # optional complete GLB in tmp/glb/
```

Use `--asset-seed N` for game asset generation, independently of `--seed` for combat.
Both default to 1993. Direct gallery options: `--model NAME`, `--seed N`,
`--clip NAME` (play continuously), `--time SECONDS` (freeze a clip), and
`--output tmp/gallery.png` (capture and exit). All 15 models share the same runtime
construction path. GLB export is an optional native `asset-export` feature and
includes materials, skins and all seven named clips; exported files are never
consumed by normal builds. See `.agents/skills/3d-assets/SKILL.md` for authoring.

Click Deploy or left-click the battlefield to start the mission. Controls: left click moves or selects a soldier; hold right click to fire; press both mouse buttons to use the selected explosive. Use the squad panel to select groups, split an individual, regroup, select grenades/rockets, deploy and retry. Keyboard equivalents: 1–3 squads, S split, J regroup, G grenades, X rockets, Enter deploy/continue, R retry. Scroll zooms; screen edges pan. The camera follows the active living squad leader near the viewing edges; manual scrolling pauses tracking briefly. F1 hides the HUD; F12 captures `tmp/battlefield.png`.

Explosives require pickups. Enemies are lethal; grenades and rockets can kill your own soldiers. Surviving soldiers carry into the next mission. Retry restores the mission-start roster. Saves are written at successful mission transitions to `saves/campaign.json`; `--fresh` starts a new roster.

## Existing editor workflow

Save the `.map` / `.spt` pair in the OpenFodder editor, then launch it directly:

```sh
cargo run -- --map '/path/My Mission.map' --tilesets data/Data/AmigaTheOne --fresh
cargo run -- --campaign '/path/Campaign.ofc' --maps '/path/maps' --tilesets data/Data/AmigaTheOne --fresh
cargo run --no-default-features --bin map-inspect -- data/Data/AmigaTheOne/mapm2.map
```

The tileset directory must contain the base/subset `.hit`, `.bht` and `.swp` files named by the map. Original tile graphics are replaced by generated 3D assets. No editor modifications are required. Unsupported sprite IDs or objectives are reported before play; the inspector can examine other environments.

## First-mission polish

The first mission now uses reviewed original tile IDs to place dense jungle thickets. Soldiers route around obstacles and follow their leader through gaps. Enemies check visibility and react according to campaign aggression. Shot effects align with aim, with upper-body firing and throwing animation, impacts and movement feedback. See [the reference and acceptance notes](docs/first-mission.md).

```sh
python3 scripts/reference_tiles.py
cargo run --release -- --fresh --benchmark --benchmark-seconds 20
cargo run -- --fresh --combat-preview
```

The benchmark writes `tmp/benchmark.json`; the combat preview writes `tmp/combat.png`. Performance numbers from offscreen container rendering do not replace host playtesting.

## Validation

```sh
cargo test --no-default-features
cargo check --all-targets
just assets-validate              # workspace contracts and WASM compatibility
cargo run -- --fresh --smoke-test
```

The smoke test captures a frame in `tmp/battlefield.png` and exits. It renders offscreen and needs no desktop display. In the Codex command sandbox, GPU devices are hidden; run GPU validation through the approved escalated execution path (see AGENTS.md). Host interactive playtesting is the final check for controls and performance.

`just assets-capture` renders `tmp/asset-library.png`.

## Data provenance

`scripts/fetch_fixtures.py` downloads 68 unmodified files at the exact OpenFodder/data revision recorded in `tests/fixtures-manifest.json`, verifies SHA-256 checksums, and stores them in ignored `data/`. Original demo data is not included in release archives. Procedural asset definitions live in `crates/models` and `crates/sound`; `assets/` retains shaders and the reviewed terrain catalog. See `docs/format.md` for the importer reference and `docs/status.md` for implementation limits.
