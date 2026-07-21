#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
destination_dir=${XDG_BIN_HOME:-"$HOME/.local/bin"}
mkdir -p "$destination_dir"

if command -v cargo >/dev/null 2>&1; then
    (cd "$project_dir" && cargo build --release)
    source_binary="$project_dir/target/release/sticky"
elif command -v rustc >/dev/null 2>&1; then
    mkdir -p "$project_dir/target/release"
    rustc --edition=2021 -C opt-level=3 -C strip=symbols \
        "$project_dir/src/main.rs" -o "$project_dir/target/release/sticky"
    source_binary="$project_dir/target/release/sticky"
else
    echo "Error: neither cargo nor rustc was found." >&2
    exit 1
fi

install -m 755 "$source_binary" "$destination_dir/sticky"
printf '\nInstalled to %s/sticky\nUsage: sticky "My note"\n' "$destination_dir"
