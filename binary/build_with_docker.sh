#!/bin/bash

# Build the Halvora AppImage using an Ubuntu 22.04 Docker container.
# This script MUST be run from the PROJECT ROOT (the parent of the binary/
# directory) so that the Docker build context contains Cargo.toml, src/,
# Font/, Halvora_Logo/, bitstamp_data/, etc.

set -e

# Resolve paths
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check if docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed. Please install it first:"
    echo "sudo apt update && sudo apt install docker.io"
    echo "sudo usermod -aG docker \$USER (then log out and back in)"
    exit 1
fi

echo "Building Halvora AppImage using Ubuntu 22.04 Docker container..."
echo "   Project root : $PROJECT_ROOT"
echo "   Binary dir   : $SCRIPT_DIR"

# Build the docker image FROM THE PROJECT ROOT so that the Dockerfile
# can COPY source files (Cargo.toml, src/, Font/, etc.) into the image.
# The -f flag points to the Dockerfile inside binary/.
docker build \
    -t halvora-builder \
    -f "$SCRIPT_DIR/Dockerfile" \
    "$PROJECT_ROOT"

# Run the container to build the AppImage.
# Mount the project root as /build so the output appears in binary/.
docker run --rm -v "$PROJECT_ROOT":/build halvora-builder

echo "Done! Your Ubuntu 22.04 compatible AppImage is in: $SCRIPT_DIR"