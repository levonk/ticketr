#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME=":latest"

case "${1:-}" in
    build)
        docker build -t "$IMAGE_NAME" .
        ;;
    run)
        shift
        docker run --rm -it "$IMAGE_NAME" "$@"
        ;;
    test)
        docker run --rm "$IMAGE_NAME" cargo test
        ;;
    *)
        echo "Usage: $0 {build|run|test} [ARGS...]"
        exit 1
        ;;
esac
