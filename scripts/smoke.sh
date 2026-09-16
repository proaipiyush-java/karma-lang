#!/usr/bin/env sh
set -eu

cargo test
cargo run --quiet -- --check examples/hello.kr
cargo run --quiet -- examples/hello.kr
cargo run --quiet -- examples/arithmetic.kr
cargo run --quiet -- examples/control_flow.kr
cargo run --quiet -- examples/functions.kr

# These files must fail static checking.
if cargo run --quiet -- --check examples/type_error.kr; then
  echo "ERROR: examples/type_error.kr unexpectedly passed type checking" >&2
  exit 1
fi

if cargo run --quiet -- --check examples/immutable_error.kr; then
  echo "ERROR: examples/immutable_error.kr unexpectedly passed type checking" >&2
  exit 1
fi

echo "Karma v0.2 smoke tests passed."
