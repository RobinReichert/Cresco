#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[ -f "$SCRIPT_DIR/cresco.local.sh" ] && . "$SCRIPT_DIR/cresco.local.sh"
export IDF_EXPORT_CMD

exec uv run --project "$SCRIPT_DIR" "$SCRIPT_DIR/scripts/cresco.py" "$@"
