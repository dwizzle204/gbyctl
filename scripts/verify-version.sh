#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

cargo_version=$(grep '^version = ' Cargo.toml | head -n1 | cut -d '"' -f2)
file_version=$(tr -d '[:space:]' < VERSION)

if [[ -z "$cargo_version" || -z "$file_version" ]]; then
  echo "missing version metadata" >&2
  exit 1
fi

if [[ "$cargo_version" != "$file_version" ]]; then
  echo "version mismatch: Cargo.toml=$cargo_version VERSION=$file_version" >&2
  exit 1
fi

if ! grep -q "## $cargo_version - " CHANGELOG.md; then
  echo "CHANGELOG.md is missing entry for version $cargo_version" >&2
  exit 1
fi

if [[ $# -gt 0 ]]; then
  expected_tag=${1#refs/tags/}
  expected_tag=${expected_tag#v}
  if [[ "$cargo_version" != "$expected_tag" ]]; then
    echo "tag version mismatch: tag=$expected_tag package=$cargo_version" >&2
    exit 1
  fi
fi

echo "version metadata verified: $cargo_version"
