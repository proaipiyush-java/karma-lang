# Karma v0.3 Local Validation Checklist

Run these commands on the development Mac before tagging v0.3.0.

## 1. Confirm toolchain

```bash
rustc --version
cargo --version
```

## 2. Run unit tests

```bash
cargo test
```

Expected: all tests pass.

## 3. Build optimized binary

```bash
cargo build --release
```

Expected binary:

```text
target/release/karma
```

## 4. Confirm version

```bash
./target/release/karma --version
```

Expected:

```text
Karma 0.3.0
```

## 5. Run smoke suite

```bash
./scripts/smoke.sh
```

Expected final line:

```text
Karma v0.3 smoke tests passed.
```

## 6. Run ownership example

```bash
./target/release/karma examples/ownership.kr
```

Expected output:

```text
Karma
Karma
Karma
Karma
temporary
42
42
```

## 7. Run reinitialization example

```bash
./target/release/karma examples/reinitialize.kr
```

Expected:

```text
first owner
new owner
```

## 8. Verify move error

```bash
./target/release/karma --check examples/move_error.kr
```

Expected: non-zero exit and a type error mentioning `use of moved value 'first'`.

## 9. Verify borrow escape error

```bash
./target/release/karma --check examples/borrow_escape_error.kr
```

Expected: non-zero exit mentioning `cannot move out of borrowed parameter 'text'`.

## 10. Verify loop ownership error

```bash
./target/release/karma --check examples/loop_move_error.kr
```

Expected: non-zero exit mentioning `loop body moves outer owned value 'message'`.

## 11. Inspect repository cleanliness

```bash
git status
```

Confirm these are not tracked:

```text
.idea/
target/
.DS_Store
.env
```

## 12. Tag only after all checks pass

```bash
git add .
git commit -m "Karma v0.3.0 ownership and resource foundation"
git tag -a v0.3.0 -m "Karma v0.3.0 - Ownership and Resource Foundation"
git push
git push origin v0.3.0
```
