#!/usr/bin/env bash
# Run through xvfb-run when no desktop display is available. Uses software Vulkan for Xvfb.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p tmp
export RUST_LOG=info,disposable_heroes=debug
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/dh-runtime}"
mkdir -p "$XDG_RUNTIME_DIR"
BINARY="${DH_BINARY:-target/x86_64-unknown-linux-gnu/release/disposable-heroes}"
"$BINARY" --fresh > tmp/native-input.log 2>&1 &
game_pid=$!
trap 'kill "$game_pid" 2>/dev/null || true' EXIT
window_id=$(xdotool search --sync --name '^Disposable Heroes$' | head -1)
xdotool windowfocus --sync "$window_id"
sleep 12
xdotool mousemove --window "$window_id" 130 643 click 1
sleep 2
xdotool mousemove --window "$window_id" 1200 850 click 1
sleep 3
xdotool mousedown 3
sleep 1
xdotool mouseup 3
xdotool key 2
xdotool key 1
xdotool key F12
sleep 3
xdotool key r
sleep 2
xdotool mousemove --window "$window_id" 1200 850 click 1
sleep 2
python3 - <<'PY'
from pathlib import Path
s=Path('tmp/native-input.log').read_text()
assert s.count('Deploying mission 1')>=2,'deploy/retry did not respond'
assert 'Movement order processed:' in s,'no movement order received'
assert 'Shot effect: Rifle' in s,'right mouse did not produce a rifle shot'
assert 'Screenshot saved' in s,'screenshot shortcut did not respond'
assert 'panicked' not in s and 'ERROR' not in s,'runtime errors; inspect tmp/native-input.log'
print('Native deploy, movement, shooting gesture, group shortcuts, screenshot and retry input exercised.')
PY
