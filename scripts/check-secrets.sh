#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  mapfile -t files < <(git ls-files)
else
  mapfile -t files < <(find . -type f \
    -not -path './target/*' \
    -not -path './.git/*' \
    -not -path './keys/*' \
    -not -path '*/node_modules/*' \
    -not -path '*/.next/*' \
    -not -path '*/dist/*')
fi

status=0
for file in "${files[@]}"; do
  case "$file" in
    ./scripts/check-secrets.sh|scripts/check-secrets.sh) continue ;;
    *keypair.json) echo "tracked program keypair: $file" >&2; status=1; continue ;;
  esac
  if [[ ! -f "$file" ]]; then
    continue
  fi
  if grep -E -n -I 'BEGIN (RSA |OPENSSH |EC )?PRIVATE KEY' "$file" >/dev/null; then
    echo "private key block in $file" >&2
    status=1
  fi
done

if [[ "$status" -ne 0 ]]; then
  exit "$status"
fi

echo "no private key blocks in scanned files"
