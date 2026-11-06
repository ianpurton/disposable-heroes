#!/usr/bin/env bash
set -euo pipefail

# Resolve project from this script's location.
ROOT="$(realpath "$(dirname "$0")/..")"
CONFIG="$ROOT/.devcontainer/devcontainer.json"

[[ -f "$CONFIG" ]] || {
    echo "Missing $CONFIG"
    exit 1
}

# Match this exact project and configuration.
mapfile -t CONTAINERS < <(
    docker ps -aq \
        --filter "label=devcontainer.local_folder=$ROOT" \
        --filter "label=devcontainer.config_file=$CONFIG"
)

if (( ${#CONTAINERS[@]} == 0 )); then
    echo "No devcontainer found for $ROOT"
    exit 0
fi

echo "Removing devcontainer(s) for $ROOT:"
docker ps -a --filter "id=${CONTAINERS[0]}" \
    --format '{{.Names}} ({{.Status}})'

for id in "${CONTAINERS[@]}"; do
    docker rm -f "$id"
done

echo "Done. Reopen the project in Zed to rebuild."
