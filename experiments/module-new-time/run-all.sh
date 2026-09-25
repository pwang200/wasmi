#!/usr/bin/env bash
# Generate every case, then time each case.wasm.
#   create_cases/   Create isolation
#   finish_cases/   Finish isolation (out-of-fuel is a valid outcome)
#   total_cases/    step-1 totals (no 100 KB pad)
#   real_total/     T-00 / T-0 / T-A / T-B — must return 1 (no OOF)
# A host crash (e.g. 20-memory-grow) is recorded and the rest continue.
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
  gen-total-a
  gen-total-b
  gen-total-b-chase
  gen-real-total-00
  gen-real-total-0
  gen-real-total-a
  gen-real-total-b
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
  rel="${wasm#"$HERE/"}"
  echo "========== $rel =========="
  log="$(mktemp)"
  if cargo run -p module-new-time --release --bin module-new-time -- "$wasm" | tee "$log"; then
    if [[ "$rel" == real_total/* ]] && ! grep -q 'finish -> 1' "$log"; then
      echo "FAIL: real_total must return 1 (no out-of-fuel)"
      failed=$((failed + 1))
    fi
  else
    echo "CRASH or error (exit $?)"
    failed=$((failed + 1))
  fi
  rm -f "$log"
done < <(find "$HERE/create_cases" "$HERE/finish_cases" "$HERE/total_cases" "$HERE/real_total" \
  -name case.wasm -not -path '*/_probe/*' | sort)

echo
echo "done. $failed case(s) crashed or failed."
