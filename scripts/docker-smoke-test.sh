#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
IMAGE_TAG=${1:-gbyctl-ci:local}

cd "$ROOT_DIR"
docker build -f docker/Dockerfile.ci -t "$IMAGE_TAG" .

docker run --rm "$IMAGE_TAG" /usr/local/bin/gbyctl --help >/dev/null

docker run --rm "$IMAGE_TAG" /usr/local/bin/gbyctl doctor --plan >/dev/null

plan_json=$(docker run --rm "$IMAGE_TAG" /usr/local/bin/gbyctl --json --plan 'disk is full')
echo "$plan_json" | jq -e '.mode == "plan-only"' >/dev/null

unsupported_json=$(docker run --rm "$IMAGE_TAG" /usr/local/bin/gbyctl --json 'write a python script')
echo "$unsupported_json" | jq -e '.mode == "out-of-scope" or .mode == "refusal"' >/dev/null

root_output=$(docker run --rm --user 0 "$IMAGE_TAG" /usr/local/bin/gbyctl doctor --plan 2>&1 || true)
echo "$root_output" | grep -q 'must not be run as root'

echo 'docker smoke test passed'
