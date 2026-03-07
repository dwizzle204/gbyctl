set shell := ["bash", "-lc"]

default:
  @just check

setup:
  cargo build

fix:
  cargo fmt --all

check:
  cargo fmt --all --check
  cargo clippy --all-targets -- -D warnings
  cargo test

build-release:
  cargo build --release

build-amd64:
  cargo build --release --target x86_64-unknown-linux-gnu

build-arm64:
  cargo build --release --target aarch64-unknown-linux-gnu

package-deb:
  cargo install cargo-deb --locked || true
  cargo deb

package-source:
  ./scripts/make-source-tarball.sh
