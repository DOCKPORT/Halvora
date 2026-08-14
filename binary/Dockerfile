# Use Ubuntu 22.04 for maximum AppImage compatibility (GLIBC 2.35)
FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies for compiling and running the iced GUI app
RUN apt-get update && apt-get install -y \
    appstream \
    binutils \
    build-essential \
    curl \
    file \
    libasound2-dev \
    libatk1.0-dev \
    libcairo2-dev \
    libdbus-1-dev \
    libegl1-mesa-dev \
    libfontconfig1-dev \
    libfuse2 \
    libglib2.0-dev \
    libgl1-mesa-dev \
    libgles2-mesa-dev \
    libgtk-3-dev \
    libharfbuzz-dev \
    libpango1.0-dev \
    libssl-dev \
    libvulkan-dev \
    libxcb-render0-dev \
    libxkbcommon-dev \
    pkg-config \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory. The build runs from the host project root, which is
# mounted here at container start time by build_with_docker.sh.
WORKDIR /build

# Copy the source trees and assets needed at compile time. Halvora embeds the
# font, app icon, and pre-compiled database into the binary via include_bytes!,
# so all of these must be present during the cargo build.
COPY Cargo.toml /build/
COPY Cargo.lock /build/
COPY src/ /build/src/
COPY Font/ /build/Font/
COPY Halvora_Logo/ /build/Halvora_Logo/
COPY bitstamp_data/ /build/bitstamp_data/

# Copy the build script (standalone file, avoids escaping issues)
COPY binary/build_appimage.sh /usr/local/bin/build_appimage.sh
RUN chmod +x /usr/local/bin/build_appimage.sh

CMD ["/usr/local/bin/build_appimage.sh"]