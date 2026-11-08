# Build the Linux game executable.
default: build

build:
    cargo build --locked --release -p disposable-heroes --bin disposable-heroes --target x86_64-unknown-linux-gnu

# Build and play; forward game arguments, for example: just play --fresh.
play *args:
    cargo run --locked --release -p disposable-heroes --bin disposable-heroes --target x86_64-unknown-linux-gnu -- {{args}}

assets-build:
    cargo build -p disposable-heroes --bin asset-gallery --target x86_64-unknown-linux-gnu

# Preview every model, or select one with: just assets-preview soldier.
assets-preview model="" seed="1993":
    cargo run -p disposable-heroes --bin asset-gallery -- --seed {{seed}} {{if model == "" { "" } else { "--model " + quote(model) }}}

# Offscreen labelled contact sheet; optional clip and time freeze an animation pose.
assets-capture output="tmp/asset-library.png" seed="1993" clip="" time="0.5":
    cargo run -p disposable-heroes --bin asset-gallery -- --seed {{seed}} --output {{quote(output)}} {{if clip == "" { "" } else { "--clip " + quote(clip) + " --time " + time }}}

assets-validate:
    cargo test --workspace --features asset-export -j 2
    cargo check -p disposable-heroes -p models -p sound --target wasm32-unknown-unknown --lib --bin disposable-heroes

# Optional interoperability output. Never used by the game or gallery.
assets-export model="" seed="1993" output="tmp/glb":
    cargo run -p disposable-heroes --features asset-export --bin asset-export -- --seed {{seed}} --output {{quote(output)}} {{if model == "" { "" } else { "--model " + quote(model) }}}

# Optional Blender scene containing the full model library, with rigs and clips.
assets-blend output="tmp/asset-library.blend" seed="1993":
    #!/usr/bin/env bash
    set -euo pipefail
    command -v blender >/dev/null || { echo 'Blender is required for assets-blend' >&2; exit 1; }
    mkdir -p tmp
    export_dir=$(mktemp -d tmp/blend-export.XXXXXX)
    trap 'rm -rf -- "$export_dir"' EXIT
    just assets-export "" {{quote(seed)}} "$export_dir"
    blender --background --factory-startup --python-exit-code 1 --python .agents/skills/3d-assets/scripts/blender_assets.py -- --input "$export_dir" --output {{quote(output)}}
