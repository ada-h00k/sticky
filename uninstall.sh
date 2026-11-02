#!/usr/bin/env sh
set -eu
DEST_DIR=${XDG_BIN_HOME:-"$HOME/.local/bin"}
if [ -f "$DEST_DIR/sticky" ]; then
  rm -- "$DEST_DIR/sticky"
  printf 'Removed %s/sticky\n' "$DEST_DIR"
else
  printf 'sticky is not installed in %s\n' "$DEST_DIR"
fi
