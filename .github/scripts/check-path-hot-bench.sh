#!/usr/bin/env bash
set -euo pipefail

BENCH_NAME="${BENCH_NAME:-path_tessellation_cache_hit_fill_100_segments}"
MAX_NS="${MAX_NS:-200}"
OUTPUT_FILE="${OUTPUT_FILE:-path_hot_bench_output.txt}"

echo "Running benchmark guard for: ${BENCH_NAME}"
echo "Threshold: ${MAX_NS} ns (median)"

cargo bench -p render-engine --bench path_pipeline -- "^${BENCH_NAME}\$" | tee "${OUTPUT_FILE}"

TIME_LINE="$(awk -v bench="${BENCH_NAME}" '
  $0 ~ "^" bench "$" { found=1; next }
  found && $0 ~ /time:[[:space:]]+\[/ { print; exit }
' "${OUTPUT_FILE}")"

if [[ -z "${TIME_LINE}" ]]; then
  echo "Failed to locate Criterion time line for benchmark '${BENCH_NAME}'."
  exit 1
fi

MEDIAN_VALUE="$(echo "${TIME_LINE}" | sed -E 's/.*\[[[:space:]]*[0-9.]+[[:space:]]+[[:alpha:]µμ]+[[:space:]]+([0-9.]+)[[:space:]]+([[:alpha:]µμ]+).*/\1/')"
MEDIAN_UNIT="$(echo "${TIME_LINE}" | sed -E 's/.*\[[[:space:]]*[0-9.]+[[:space:]]+[[:alpha:]µμ]+[[:space:]]+([0-9.]+)[[:space:]]+([[:alpha:]µμ]+).*/\2/')"

if [[ -z "${MEDIAN_VALUE}" || -z "${MEDIAN_UNIT}" ]]; then
  echo "Failed to parse median benchmark value from: ${TIME_LINE}"
  exit 1
fi

MEDIAN_NS="$(awk -v v="${MEDIAN_VALUE}" -v u="${MEDIAN_UNIT}" '
  BEGIN {
    if (u == "ns") print v;
    else if (u == "us" || u == "µs" || u == "μs") print v * 1000.0;
    else if (u == "ms") print v * 1000000.0;
    else if (u == "s") print v * 1000000000.0;
    else exit 2;
  }
')"

if [[ -z "${MEDIAN_NS}" ]]; then
  echo "Failed to convert benchmark unit '${MEDIAN_UNIT}' to ns."
  exit 1
fi

echo "Median: ${MEDIAN_VALUE} ${MEDIAN_UNIT} (${MEDIAN_NS} ns)"

awk -v value="${MEDIAN_NS}" -v max="${MAX_NS}" '
  BEGIN {
    if (value <= max) {
      printf("PASS: %.3f ns <= %.3f ns\n", value, max);
      exit 0;
    }
    printf("FAIL: %.3f ns > %.3f ns\n", value, max);
    exit 1;
  }
'
