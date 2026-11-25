#!/usr/bin/env bash
# Fails if the generated code in the repository is stale.
#
# The script runs `buf generate` and then compares the result with the
# committed files. Run it from any directory in the repository.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

generated_dir="crates/nomux-proto/src/generated"

buf generate

# Compare the working tree with the index: changed and deleted files come
# from `git diff`, new files come from `git ls-files --others`.
changed="$(git diff --name-only -- "$generated_dir")"
untracked="$(git ls-files --others --exclude-standard -- "$generated_dir")"
if [ -n "$changed" ] || [ -n "$untracked" ]; then
  echo "error: generated code is stale. Run 'buf generate' and commit the result." >&2
  printf '%s\n' $changed $untracked >&2
  git --no-pager diff -- "$generated_dir" >&2
  exit 1
fi

echo "generated code is up to date"
