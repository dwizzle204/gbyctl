#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

cd "$ROOT_DIR"

deb_path=$(find target/debian -maxdepth 1 -type f -name 'gbyctl_*_amd64.deb' | head -n1)
if [[ -z "${deb_path:-}" ]]; then
  echo "expected a built Debian package under target/debian" >&2
  exit 1
fi

docker run --rm \
  -v "$ROOT_DIR/target/debian:/packages:ro" \
  ubuntu:24.04 \
  bash -euxo pipefail -c '
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y --no-install-recommends ca-certificates jq libgcc-s1 /packages/*.deb

    useradd --create-home --uid 10001 gbyctl

    su -s /bin/bash -c "GBYCTL_EPHEMERAL=1 gbyctl --help >/dev/null" gbyctl
    su -s /bin/bash -c "GBYCTL_EPHEMERAL=1 gbyctl --plan \"why is my server slow\" >/dev/null" gbyctl

    plan_json=$(su -s /bin/bash -c "GBYCTL_EPHEMERAL=1 gbyctl --json --plan \"disk is full\"" gbyctl)
    echo "$plan_json" | jq -e ".mode == \"plan-only\"" >/dev/null

    unsupported_json=$(su -s /bin/bash -c "GBYCTL_EPHEMERAL=1 gbyctl --json \"write a python script\"" gbyctl || true)
    echo "$unsupported_json" | jq -e ".mode == \"out_of_scope\" or .mode == \"refusal\"" >/dev/null

    root_output=$(GBYCTL_EPHEMERAL=1 gbyctl --plan "why is my server slow" 2>&1 || true)
    echo "$root_output" | grep -q "must not be run as root"
  '

echo 'docker deb smoke test passed'
