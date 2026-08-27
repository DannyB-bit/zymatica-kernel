#!/usr/bin/env bash
set -euo pipefail

MODEL_DIR="${1:-${ZYMATICA_MODEL_DIR:-}}"
Q8_CACHE_DIR="${2:-${ZYMATICA_Q8_CACHE_DIR:-${MODEL_DIR}/.zymatica-q8-cache}}"
ENGINE="${ZYMATICA_ENGINE:-auto}"
PROMPT_IDS="${ZYMATICA_PROMPT_IDS:-2}"
NEW_TOKENS="${ZYMATICA_NEW_TOKENS:-32}"
PASSES="${ZYMATICA_PASSES:-3}"

if [[ -z "${MODEL_DIR}" ]]; then
  echo "usage: scripts/pi_field_bench.sh /path/to/gemma-4-E2B-it [/path/to/q8-cache]" >&2
  echo "or set ZYMATICA_MODEL_DIR=/path/to/gemma-4-E2B-it" >&2
  exit 2
fi

cargo build --release

./target/release/zymatica-engine pi-bench \
  --model-dir "${MODEL_DIR}" \
  --engine "${ENGINE}" \
  --q8-cache-dir "${Q8_CACHE_DIR}" \
  --prompt-ids "${PROMPT_IDS}" \
  --new-tokens "${NEW_TOKENS}" \
  --passes "${PASSES}"
