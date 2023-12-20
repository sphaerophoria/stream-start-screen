#!/usr/bin/env bash

set -ex

cargo fmt --check
cargo clippy -- -Dwarnings -Adead_code
cargo test
