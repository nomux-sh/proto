#!/usr/bin/env bash
# Fails if the generated code in the repository is stale.
#
# The script runs `buf generate` and then compares the result with the
# committed files. Run it from any directory in the repository.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

generated_dir="crates/nomux-proto/src/generated"

buf generate

stale="$(git status --porcelain -- "$generated_dir")"
if [ -n "$stale" ]; then
  echo "error: generated code is stale. Run 'buf generate' and commit the result." >&2
  printf '%s\n' "$stale" >&2
  git --no-pager diff -- "$generated_dir" >&2
  exit 1
fi

echo "generated code is up to date"
