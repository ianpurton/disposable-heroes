# Development environment

You are running in a Linux devcontainer set up for Bevy development with GPU access.

Read [README Layout](README.md#layout) for repository architecture and asset ownership,
and [Run](README.md#run), [Tasks](README.md#tasks) and
[Validation](README.md#validation) for target, commands and playtesting guidance.

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

Use the repository-local `3d-assets` skill for models, sounds and asset tooling.
See [README Layout](README.md#layout) for authoritative asset sources and
[Tasks](README.md#tasks) for the shared game/gallery/export pipeline.
Do not use exported GLBs as source or introduce a Blender/Python build dependency.
