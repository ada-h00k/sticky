#!/usr/bin/env sh
set -eu
REPO="ada-h00k/sticky"
DEST_DIR=\${XDG_BIN_HOME:-"$HOME/.local/bin"}
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ASSET="sticky-linux-x86_64" ;;
  aarch64|arm64) ASSET="sticky-linux-aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac
mkdir -p "$DEST_DIR"
TMP_FILE=$(mktemp)
trap 'rm -f "$TMP_FILE"' EXIT HUP INT TERM
downloaded=0
if command -v curl >/dev/null 2>&1 && command -v python3 >/dev/null 2>&1; then
  RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/$REPO/releases?per_page=1" || true)
  if [ -n "$RELEASE_JSON" ]; then
    ASSET_URL=$(printf '%s' "$RELEASE_JSON" | python3 -c '
import json, sys
wanted = sys.argv[1]
try:
    for release in json.load(sys.stdin):
        for asset in release.get("assets", []):
            if asset.get("name") == wanted:
                print(asset.get("browser_download_url", ""))
                raise SystemExit
except (ValueError, TypeError):
    pass
' "$ASSET")
    if [ -n "$ASSET_URL" ] && curl -fL "$ASSET_URL" -o "$TMP_FILE"; then downloaded=1; fi
  fi
fi
if [ "$downloaded" -eq 0 ]; then
  PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
  if command -v cargo >/dev/null 2>&1; then
    (cd "$PROJECT_DIR" && cargo build --release)
    cp "$PROJECT_DIR/target/release/sticky" "$TMP_FILE"
  elif command -v rustc >/dev/null 2>&1; then
    rustc --edition=2021 -C opt-level=3 -C strip=symbols "$PROJECT_DIR/src/main.rs" -o "$TMP_FILE"
  else
    echo "No matching release binary found and Rust is not installed." >&2
    echo "Try again after the next CI release, or install Cargo/rustc." >&2
    exit 1
  fi
fi
install -m 755 "$TMP_FILE" "$DEST_DIR/sticky"
printf '\nInstalled sticky to %s/sticky\n' "$DEST_DIR"
case ":$PATH:" in *":$DEST_DIR:"*) ;; *) printf 'Add to PATH: export PATH="%s:$PATH"\n' "$DEST_DIR" ;; esac
printf 'Usage: sticky "My note"\n'
