#!/bin/sh
# Prints the body of a version's section in CHANGELOG.md (without its
# heading), for use as GitHub release notes.
#
# Usage: changelog-section.sh <version> [changelog-file]
#
# Fails if the section is missing, still marked "Unreleased", or empty.
set -eu

version="$1"
file="${2:-CHANGELOG.md}"
heading_prefix="## [$version]"

heading=$(awk -v h="$heading_prefix" 'index($0, h) == 1 { print; exit }' "$file")
if [ -z "$heading" ]; then
    echo "error: no '$heading_prefix' section in $file" >&2
    exit 1
fi
case "$heading" in
    *Unreleased*)
        echo "error: section '$heading' in $file is not dated" >&2
        exit 1
        ;;
esac

# Section body: up to the next version heading or the link definitions.
body=$(awk -v h="$heading_prefix" '
    index($0, h) == 1 { on = 1; next }
    on && (/^## \[/ || /^\[[^]]*\]: /) { exit }
    on { print }
' "$file")

if [ -z "$(printf '%s' "$body" | tr -d '[:space:]')" ]; then
    echo "error: section '$heading' in $file is empty" >&2
    exit 1
fi
printf '%s\n' "$body"
