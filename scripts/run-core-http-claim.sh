#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo test --test core_http_claim
cargo test --test core_http_blackbox
