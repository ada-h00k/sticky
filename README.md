# sticky

A small Rust CLI for KDE Plasma. The intended usage is:

```bash
sticky "Text to put in the note"
```

The program creates a file in the directory used by Plasma,
`$XDG_DATA_HOME/plasma_notes` (by default `~/.local/share/plasma_notes`), and
then adds the preinstalled `org.kde.plasma.notes` widget to a panel via Plasma
scripting.

## Requirements

- Linux with KDE Plasma 6 (Plasma 5 will likely work as well)
- The **Sticky Note** widget (`org.kde.plasma.notes`)
- One of the following D-Bus tools: `qdbus6`, `qdbus`, `gdbus`, or `busctl`
- Rust/Cargo for compilation

## Installation

The easiest way:

```bash
./install.sh
```

The script uses `cargo`, falls back to a direct `rustc` build if necessary, and
installs the binary to `~/.local/bin/sticky` (or `$XDG_BIN_HOME/sticky`). Make
sure that `~/.local/bin` is included in your `PATH`.

Alternatively, install it with Cargo:

```bash
cargo install --path .
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

## Development

```bash
cargo test
cargo build --release
```

The resulting binary is located at `target/release/sticky`.

## Technical Note

The note text is stored as rich-text HTML understood by Qt. This ensures that
special characters and multiline text are displayed correctly without
requiring the user to enter HTML.

## License

This project is licensed under the **GNU Affero General Public License v3.0 only**
(`AGPL-3.0-only`). See [`LICENSE`](LICENSE) for the full license text.
