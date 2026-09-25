#!/usr/bin/env bash
# Generate every Create/Finish case, then time each case.wasm.
# A case that crashes the host (e.g. 20-memory-grow) is recorded and the rest continue.
set -u
set -o pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

GENS=(
  gen-01-active-element
  gen-02-br-table
  gen-03-func-spam
  gen-04-nested-blocks
  gen-05-fat-data
  gen-06-many-locals
  gen-07-combo
  gen-finish-00-baseline
  gen-finish-01-hot-call-locals
  gen-finish-02-call-indirect
  gen-finish-03-div-u
  gen-finish-04-div-u64
  gen-finish-05-load-same
  gen-finish-06-load-stride-4
  gen-finish-07-load-stride-64
  gen-finish-08-load-stride-4096
  gen-finish-09-load-stride-4160
  gen-finish-10-br-table
  gen-finish-11-nested-blocks
  gen-finish-12-many-callees
  gen-finish-13-hot-call-0-locals
  gen-finish-16-unroll
  gen-finish-17-nest
  gen-finish-18-recurse
  gen-finish-20-memory-grow
)

echo "======== generate ========"
for bin in "${GENS[@]}"; do
  echo "=== $bin ==="
  cargo run -p module-new-time --release --bin "$bin"
done

echo
echo "======== time ========"
failed=0
while IFS= read -r wasm; do
  echo "========== ${wasm#"$HERE/"} =========="
  if cargo run -p module-new-time --release --bin module-new-time -- "$wasm"; then
    :
  else
    echo "CRASH or error (exit $?)"
    failed=$((failed + 1))
  fi
done < <(find "$HERE/create_cases" "$HERE/finish_cases" -name case.wasm | sort)

echo
echo "done. $failed case(s) crashed or failed."
