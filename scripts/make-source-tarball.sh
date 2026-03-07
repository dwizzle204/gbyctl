#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
VERSION=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -n1 | cut -d '"' -f2)
OUT="$ROOT_DIR/gbyctl-${VERSION}.tar.gz"

cd "$ROOT_DIR"
tar --warning=no-file-changed --exclude='./target' --exclude='./.git' -czf "$OUT" .
echo "created $OUT"
