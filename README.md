# sticky

A small Rust CLI for creating KDE Plasma sticky notes from the terminal.

```bash
sticky "Text to put in the note"
```

The program stores notes in `$XDG_DATA_HOME/plasma_notes` (by default
`~/.local/share/plasma_notes`) and adds the preinstalled
`org.kde.plasma.notes` widget to a Plasma panel through the Plasma scripting
interface.

## Requirements

- Linux with KDE Plasma (Plasma 6 recommended).
- The **Sticky Note** Plasma widget (`org.kde.plasma.notes`).
- At runtime, one of `qdbus6`, `qdbus`, `gdbus`, or `busctl`.
- For the one-line installer: `curl` and `python3`.

This is a KDE/Plasma application. Fedora Atomic desktops running GNOME are
not supported because GNOME does not provide the Plasma Notes widget or its
scripting interface.

## Install on Fedora Atomic and other Linux systems

The installer downloads the latest architecture-matched binary release and
installs it to `~/.local/bin`. It does not use `rpm-ostree`, `dnf`, or
modify the immutable host system.

```bash
curl -fsSL https://raw.githubusercontent.com/ada-h00k/sticky/main/install.sh | sh
```

Prebuilt release binaries are provided for x86_64 and aarch64.

If you have cloned the repository, run:

```bash
./install.sh
```

If `~/.local/bin` is not already in your PATH, add it:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Uninstall

From a clone:

```bash
./uninstall.sh
```

Or remove the binary manually:

```bash
rm -f "${XDG_BIN_HOME:-$HOME/.local/bin}/sticky"
```

## Usage

```bash
sticky "Buy milk"
sticky "Line 1
Line 2"
echo "Text from stdin" | sticky -
```

The first available panel is used. When multiple panels are present:

```bash
sticky --screen 1 "Note on screen 1"
sticky --panel 3 "Note in the panel with ID 3"
```

Additional options:

```text
--color COLOR   white, black, red, orange, yellow, green, blue, pink,
                translucent, or translucent-light
--pin-open      Keep the panel note popup open
```

## Automated releases

GitHub Actions builds native x86_64 and aarch64 binaries for every pushed
commit and publishes them as prereleases named `commit-<full-sha>`. The
installer automatically downloads the newest published build.

## Development

```bash
cargo test
cargo build --release
```

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
