#!/usr/bin/env bash

set -euo pipefail

IMAGE_NAME="damingerdai/rammal"
IMAGE_TAG="${1:-alpine-latest}"

echo "Building Docker image: ${IMAGE_NAME}:${IMAGE_TAG}"

GIT_DESCRIBE=$(git describe --always --dirty 2>/dev/null || echo "unknown")
GIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
GIT_COMMIT_DATE=$(git log -1 --format=%cI 2>/dev/null || echo "unknown")

docker build \
  --build-arg VERGEN_GIT_DESCRIBE="${GIT_DESCRIBE}" \
  --build-arg VERGEN_GIT_SHA="${GIT_SHA}" \
  --build-arg VERGEN_GIT_COMMIT_DATE="${GIT_COMMIT_DATE}" \
  -f Dockerfile.alpine \
  -t "${IMAGE_NAME}:${IMAGE_TAG}" \
  .

echo "Build completed:"
docker images "${IMAGE_NAME}:${IMAGE_TAG}"
