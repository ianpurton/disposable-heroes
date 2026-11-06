FROM purtontech/rust-on-nails-devcontainer:1.3.46 AS development

ENV PATH="/usr/local/cargo/bin:${PATH}"

COPY ps1.bash .
RUN cat ps1.bash >> ~/.bashrc && sudo rm ps1.bash

COPY bash_aliases .
RUN cat bash_aliases >> ~/.bash_aliases && sudo rm bash_aliases

# Enable our git hooks and set the permisisons on docker sock.
RUN echo 'git config core.hooksPath /workspace/.devcontainer/.githooks' >> ~/.bashrc

RUN cargo binstall --no-confirm --locked typst-cli

ARG USERNAME=vscode

# Bevy Linux development and an optional virtual display for smoke tests.
RUN sudo apt-get update && sudo apt-get install -y --no-install-recommends \
    pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-0 \
    libx11-dev libxcursor1 libxrandr2 libxi6 \
    libvulkan1 mesa-vulkan-drivers clang lld vulkan-tools xvfb xauth xdotool \
    && sudo rm -rf /var/lib/apt/lists/*

    # Git LFS for Blender source files
    RUN sudo apt-get update && \
        sudo apt-get install -y --no-install-recommends \
            git-lfs curl xz-utils libegl1 libgl1 && \
        sudo rm -rf /var/lib/apt/lists/*

    # Blender for headless .blend -> GLB asset generation
    ARG BLENDER_VERSION=5.2.3

    RUN curl -fL \
          "https://download.blender.org/release/Blender5.2/blender-${BLENDER_VERSION}-linux-x64.tar.xz" \
          -o /tmp/blender.tar.xz && \
        sudo mkdir -p /opt/blender && \
        sudo tar -xJf /tmp/blender.tar.xz \
          -C /opt/blender --strip-components=1 && \
        sudo ln -s /opt/blender/blender /usr/local/bin/blender && \
        rm /tmp/blender.tar.xz

    # Verify installation
    RUN blender --version | head -n 1 && git lfs version
