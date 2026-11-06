# Development environment

You are running in a Linux devcontainer set up for Bevy development with GPU access.

Build the game for `x86_64-unknown-linux-gnu`, matching the user's Linux host.

The user runs the game on the Linux host for interactive playtesting.

Installed tools: Rust, Blender, Vulkan.

## Visual test output

Save test screenshots, rendered previews, and other test images in `/workspace/tmp`
(`./tmp`), so the user can inspect them. Link useful images in progress updates.

## GPU checks

The command sandbox hides `/dev/dri` and the supplementary render-device group.
Sandboxed `vulkaninfo --summary` may therefore report only the CPU renderer
(`llvmpipe`) even when devcontainer GPU passthrough is working.

Run hardware GPU diagnostics and GPU-dependent validation through the approved
`require_escalated` execution path. Do not change device permissions or rebuild
the devcontainer based only on a sandboxed GPU check.

Verified outside the sandbox: `/dev/dri/renderD128` is accessible through group
`992`, and Vulkan detects `AMD Radeon Graphics (RADV GFX1150)` using Mesa RADV.
The host initialization script generates the supplementary group dynamically;
do not hard-code this machine's group ID into the configuration.

GPU access does not provide a desktop display. Interactive playtesting remains
on the Linux host; container rendering checks need a headless or virtual display.

## Procedural model pipeline

Python definitions in `assets/models/` are the source of truth. Generated GLBs live
in `assets/glb/`. Use the repository-local `3d-assets` skill when changing
models or the export pipeline. Its runner is invoked by `just glbs` (all models),
`just glb soldier` (one model), and `just blends` (also save inspection snapshots
under `tmp/blender/`). Do not hand-edit generated GLBs or treat snapshots as source.
