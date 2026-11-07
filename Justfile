# Generate assets in memory and compile the gallery.
default: assets-build

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
