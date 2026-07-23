#!/usr/bin/env sh
set -eu

MODE="${1:-update}"

case "$MODE" in
  update|check) ;;
  *)
    echo "Usage: $0 [update|check]" >&2
    exit 2
    ;;
esac

ROOT="$(
	CDPATH=
	cd -- "$(dirname -- "$0")/../.." || exit
	pwd
)"
cd "$ROOT"

VERSION="$(
  cargo metadata --no-deps --format-version=1 \
    | tr '{' '\n' \
    | sed -n '/"name":"tildr"/s/.*"version":"\([^"]*\)".*/\1/p' \
    | head -n 1
)"

if [ -z "$VERSION" ]; then
  echo "Could not resolve tildr version from cargo metadata." >&2
  exit 1
fi

changed=0
failed=0

for file in docs/site/*.md; do
  if ! grep -Eq '^version: ".*"$' "$file"; then
    echo "Missing version front matter: $file" >&2
    failed=1
    continue
  fi

  if ! grep -Eq '^permalink: /tildr/documentation/[^/]+(/.*)$' "$file"; then
    echo "Missing or unsupported documentation permalink: $file" >&2
    failed=1
    continue
  fi

  tmp="$(mktemp "${TMPDIR:-/tmp}/tildr-docs-version.XXXXXX")"

  sed -E \
    -e "s|^version: \".*\"$|version: \"$VERSION\"|" \
    -e "s|^(permalink: /tildr/documentation/)[^/]+(/.*)$|\1$VERSION\2|" \
    "$file" > "$tmp"

  if ! cmp -s "$file" "$tmp"; then
    changed=1

    if [ "$MODE" = "check" ]; then
      echo "Outdated docs version: $file" >&2
      rm -f "$tmp"
    else
      mv "$tmp" "$file"
    fi
  else
    rm -f "$tmp"
  fi
done

if [ "$failed" -ne 0 ]; then
  exit 1
fi

if [ "$MODE" = "check" ] && [ "$changed" -ne 0 ]; then
  echo "Run: make docs-version" >&2
  exit 1
fi

if [ "$MODE" = "update" ]; then
  echo "docs/site version synced to $VERSION"
fi
