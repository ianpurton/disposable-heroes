#!/usr/bin/env bash
# Build the same WebGPU package as releases, without publishing it.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

if [[ ! -d data/Data/AmigaTheOne ]]; then
    echo 'Game data is missing. Run python3 scripts/fetch_fixtures.py first.' >&2
    exit 1
fi
version="$(cargo tree --locked -p disposable-heroes --target wasm32-unknown-unknown -i wasm-bindgen --prefix none | sed -n 's/^wasm-bindgen v\([0-9][0-9.]*\).*$/\1/p' | head -n 1)"
test -n "$version" || { echo 'Cannot find wasm-bindgen dependency' >&2; exit 1; }
if command -v wasm-bindgen >/dev/null && [[ "$(wasm-bindgen --version)" == "wasm-bindgen $version" ]]; then
    bindgen="$(command -v wasm-bindgen)"
else
    case "$(uname -s)/$(uname -m)" in
        Linux/x86_64) cli_target=x86_64-unknown-linux-musl ;;
        Linux/aarch64) cli_target=aarch64-unknown-linux-musl ;;
        *) echo "Install wasm-bindgen CLI $version for this host first." >&2; exit 1 ;;
    esac
    bundle="wasm-bindgen-${version}-${cli_target}"
    install_dir="$PWD/target/tools/$bundle"
    bindgen="$install_dir/$bundle/wasm-bindgen"
    if [[ ! -x "$bindgen" ]]; then
        archive="${bundle}.tar.gz"
        release_url="https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${version}"
        mkdir -p "$install_dir/gh-release"
        curl --fail --location --silent --show-error --retry 3 \
            "$release_url/$archive" --output "$install_dir/gh-release/$archive"
        curl --fail --location --silent --show-error --retry 3 \
            "$release_url/$archive.sha256sum" --output "$install_dir/$archive.sha256sum"
        (cd "$install_dir" && sha256sum --check "$archive.sha256sum")
        tar -xzf "$install_dir/gh-release/$archive" -C "$install_dir"
    fi
fi
test "$("$bindgen" --version)" = "wasm-bindgen $version"

cargo build --locked --release --target wasm32-unknown-unknown -p disposable-heroes --bin disposable-heroes
# Only replace the generated output after compilation succeeds.
rm -rf -- dist/web
mkdir -p dist/web
"$bindgen" --target web --out-dir dist/web --out-name disposable_heroes \
    target/wasm32-unknown-unknown/release/disposable-heroes.wasm
cp web/index.html dist/web/index.html
cp -a assets dist/web/assets
cp -a data dist/web/data
# Match the browser importer's URLs and the release package.
find dist/web/data -type f -name '*.swp' -exec mv -- '{}' '{}.bin' \;
printf 'Web build ready: dist/web/\nServe: python3 -m http.server 8000 --directory dist/web\n'
