#!/usr/bin/env sh
set -eu

cargo test
cargo run --quiet -- examples/hello.kr
cargo run --quiet -- examples/arithmetic.kr
cargo run --quiet -- examples/control_flow.kr
cargo run --quiet -- examples/functions.kr
