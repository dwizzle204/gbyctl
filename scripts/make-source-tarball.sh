#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
VERSION=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -n1 | cut -d '"' -f2)
OUT="$ROOT_DIR/gbyctl-${VERSION}.tar.gz"
TMP_OUT=$(mktemp "/tmp/gbyctl-${VERSION}.XXXXXX.tar.gz")
trap 'rm -f "$TMP_OUT"' EXIT

cd "$ROOT_DIR"
tar \
  --exclude="./target" \
  --exclude="./.git" \
  --exclude="./gbyctl-*.tar.gz" \
  -czf "$TMP_OUT" .
mv "$TMP_OUT" "$OUT"
echo "created $OUT"
