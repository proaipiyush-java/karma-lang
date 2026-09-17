#!/usr/bin/env sh
set -eu

cargo test
cargo run --quiet -- --check examples/hello.kr
cargo run --quiet -- examples/hello.kr
cargo run --quiet -- examples/arithmetic.kr
cargo run --quiet -- examples/control_flow.kr
cargo run --quiet -- examples/functions.kr
cargo run --quiet -- examples/ownership.kr
cargo run --quiet -- examples/reinitialize.kr

# These files must fail static checking.
for file in \
  examples/type_error.kr \
  examples/immutable_error.kr \
  examples/move_error.kr \
  examples/borrow_escape_error.kr \
  examples/loop_move_error.kr
do
  if cargo run --quiet -- --check "$file"; then
    echo "ERROR: $file unexpectedly passed static checking" >&2
    exit 1
  fi
done

echo "Karma v0.3 smoke tests passed."
