#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "[1/4] C++20 configure/build"
cmake -S "$ROOT/cpp" -B "$ROOT/cpp/build" -DCMAKE_BUILD_TYPE=Release
cmake --build "$ROOT/cpp/build" -j

echo "[2/4] C++20 tests"
ctest --test-dir "$ROOT/cpp/build" --output-on-failure

echo "[3/4] Independent golden vectors"
python3 "$ROOT/tools/reference_vectors.py" > "$ROOT/.golden.tmp.json"
diff -u "$ROOT/GOLDEN_VECTORS.json" "$ROOT/.golden.tmp.json"
rm "$ROOT/.golden.tmp.json"

echo "[4/4] Rust format/lint/test"
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo/rustc not installed. Rust verification is mandatory for a full release." >&2
  exit 2
fi
(
  cd "$ROOT/rust"
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test --all-targets
)

echo "Z-SPAR FULL VERIFICATION PASS"
