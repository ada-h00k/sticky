use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Output};

const VERSION: &str = match option_env!("CARGO_PKG_VERSION") {
    Some(version) => version,
    None => "0.1.0",
};
const PLASMOID_ID: &str = "org.kde.plasma.notes";
const VALID_COLORS: &[&str] = &[
    "white",
    "black",
    "red",
    "orange",
    "yellow",
    "green",
    "blue",
    "pink",
    "translucent",
    "translucent-light",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    text: String,
    panel_id: Option<u32>,
    screen: Option<i32>,
    color: String,
    pin_open: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("sticky: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args(env::args_os().skip(1).collect())?;
    let note_id = generate_uuid_v4()?;
    let note_path = create_note_file(&note_id, &options.text)?;
    let script = build_plasma_script(&note_id, &options);

    match evaluate_plasma_script(&script) {
        Ok(widget_id) => {
            if widget_id == "unknown" {
                println!("Sticky note created.");
            } else {
                println!("Sticky note created (widget ID: {widget_id}).");
            }
            Ok(())
        }
        Err(error) => {
            // If Plasma rejected the request, avoid leaving an orphaned note file.
            // A partially created widget is possible only if Plasma failed after addWidget().
            let _ = fs::remove_file(&note_path);
            Err(error)
        }
    }
}

fn parse_args(args: Vec<OsString>) -> Result<Options, String> {
    let mut panel_id = None;
    let mut screen = None;
    let mut color = String::from("yellow");
    let mut pin_open = false;
    let mut text_parts = Vec::new();
    let mut positional_only = false;
    let mut index = 0;

    while index < args.len() {
        let arg = args[index].to_string_lossy();

        if positional_only {
            text_parts.push(arg.into_owned());
            index += 1;
            continue;
        }

        match arg.as_ref() {
            "--" => {
                positional_only = true;
                index += 1;
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("sticky {VERSION}");
                std::process::exit(0);
            }
            "--pin-open" => {
                pin_open = true;
                index += 1;
            }
            "--panel" => {
                let value = next_value(&args, index, "--panel")?;
                panel_id = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("invalid panel ID: {value}"))?,
                );
                index += 2;
            }
            "--screen" => {
                let value = next_value(&args, index, "--screen")?;
                screen = Some(
                    value
                        .parse::<i32>()
                        .map_err(|_| format!("invalid screen index: {value}"))?,
                );
                index += 2;
            }
            "--color" => {
                let value = next_value(&args, index, "--color")?;
                if !VALID_COLORS.contains(&value.as_str()) {
                    return Err(format!(
                        "unknown color '{value}'. Allowed values: {}",
                        VALID_COLORS.join(", ")
                    ));
                }
                color = value;
                index += 2;
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option: {value}\n\nSee: sticky --help"));
            }
            _ => {
                text_parts.push(arg.into_owned());
                index += 1;
            }
        }
    }

    if panel_id.is_some() && screen.is_some() {
        return Err("--panel and --screen cannot be used together".into());
    }

    let text = if text_parts.len() == 1 && text_parts[0] == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("could not read from stdin: {error}"))?;
        input
    } else {
        text_parts.join(" ")
    };

    if text.is_empty() {
        return Err("no note text provided\n\nExample: sticky \"Buy milk\"".into());
    }

    Ok(Options {
        text,
        panel_id,
        screen,
        color,
        pin_open,
    })
}

fn next_value(args: &[OsString], index: usize, option: &str) -> Result<String, String> {
    args.get(index + 1)
        .map(|value| value.to_string_lossy().into_owned())
        .ok_or_else(|| format!("{option} requires a value"))
}

fn print_help() {
    println!(
        "sticky {VERSION}\n\
         Creates a KDE Plasma sticky note in a panel.\n\n\
         USAGE:\n\
           sticky [OPTIONS] \"TEXT\"\n\
           echo \"multiline\" | sticky -\n\n\
         OPTIONS:\n\
           --panel ID       Use a specific Plasma panel\n\
           --screen INDEX   Use the first panel on this screen\n\
           --color COLOR    Note color (default: yellow)\n\
           --pin-open       Keep the panel note popup open\n\
           -h, --help       Show help\n\
           -V, --version    Show version\n\n\
         Without --panel or --screen, the first available panel is used."
    );
}

fn data_home() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    let home = env::var_os("HOME").ok_or("neither XDG_DATA_HOME nor HOME is set")?;
    Ok(PathBuf::from(home).join(".local/share"))
}

fn create_note_file(note_id: &str, text: &str) -> Result<PathBuf, String> {
    let notes_dir = data_home()?.join("plasma_notes");
    fs::create_dir_all(&notes_dir).map_err(|error| {
        format!(
            "could not create note directory '{}': {error}",
            notes_dir.display()
        )
    })?;

    let path = notes_dir.join(note_id);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("could not create note file '{}': {error}", path.display()))?;

    let rich_text = plain_text_to_qt_html(text);
    file.write_all(rich_text.as_bytes())
        .map_err(|error| format!("could not write note file '{}': {error}", path.display()))?;

    Ok(path)
}

fn plain_text_to_qt_html(text: &str) -> String {
    let mut html = String::with_capacity(text.len() + 64);
    html.push_str("<html><body><p>");

    for ch in text.chars() {
        match ch {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\'' => html.push_str("&#39;"),
            '\n' => html.push_str("<br/>"),
            '\r' => {}
            '\t' => html.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            _ => html.push(ch),
        }
    }

    html.push_str("</p></body></html>");
    html
}

fn generate_uuid_v4() -> Result<String, String> {
    let mut bytes = [0_u8; 16];
    let mut random = fs::File::open("/dev/urandom")
        .map_err(|error| format!("could not open /dev/urandom: {error}"))?;
    random
        .read_exact(&mut bytes)
        .map_err(|error| format!("could not read random data: {error}"))?;

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let mut uuid = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            uuid.push('-');
        }
        write!(&mut uuid, "{byte:02x}").expect("writing to String cannot fail");
    }
    Ok(uuid)
}

fn build_plasma_script(note_id: &str, options: &Options) -> String {
    let panel_selector = if let Some(panel_id) = options.panel_id {
        format!(
            "panel = panelById({panel_id});\n\
             if (!panel) return 'STICKY_ERROR:Panel {panel_id} was not found';"
        )
    } else if let Some(screen) = options.screen {
        format!(
            "for (let i = 0; i < panelIds.length; ++i) {{\n\
                 const candidate = panelById(panelIds[i]);\n\
                 if (candidate && candidate.screen === {screen}) {{ panel = candidate; break; }}\n\
             }}\n\
             if (!panel) return 'STICKY_ERROR:No panel found on screen {screen}';"
        )
    } else {
        "for (let i = 0; i < panelIds.length; ++i) {\n\
             const candidate = panelById(panelIds[i]);\n\
             if (candidate) { panel = candidate; break; }\n\
         }\n\
         if (!panel) return 'STICKY_ERROR:No Plasma panel found';"
            .to_string()
    };

    format!(
        "(() => {{\n\
             try {{\n\
                 let panel = null;\n\
                 {panel_selector}\n\
                 const widget = panel.addWidget('{PLASMOID_ID}');\n\
                 if (!widget) return 'STICKY_ERROR:Could not create the Sticky Note widget';\n\
                 widget.currentConfigGroup = ['General'];\n\
                 widget.writeConfig('noteId', '{note_id}');\n\
                 widget.writeConfig('color', '{}');\n\
                 widget.writeConfig('pinOpen', {});\n\
                 widget.reloadConfig();\n\
                 return 'STICKY_OK:' + widget.id;\n\
             }} catch (error) {{\n\
                 return 'STICKY_ERROR:' + String(error);\n\
             }}\n\
         }})()",
        options.color,
        if options.pin_open { "true" } else { "false" }
    )
}

fn evaluate_plasma_script(script: &str) -> Result<String, String> {
    let backends = [
        DbusBackend::Qdbus("qdbus6"),
        DbusBackend::Qdbus("qdbus"),
        DbusBackend::Gdbus,
        DbusBackend::Busctl,
    ];

    for backend in backends {
        match backend.run(script) {
            BackendResult::NotInstalled => continue,
            BackendResult::Finished(output) => return parse_dbus_output(backend.name(), output),
        }
    }

    Err("no D-Bus tool found (tried: qdbus6, qdbus, gdbus, busctl)".into())
}

#[derive(Clone, Copy)]
enum DbusBackend {
    Qdbus(&'static str),
    Gdbus,
    Busctl,
}

enum BackendResult {
    NotInstalled,
    Finished(Output),
}

impl DbusBackend {
    fn name(self) -> &'static str {
        match self {
            Self::Qdbus(name) => name,
            Self::Gdbus => "gdbus",
            Self::Busctl => "busctl",
        }
    }

    fn run(self, script: &str) -> BackendResult {
        let result = match self {
            Self::Qdbus(program) => Command::new(program)
                .args([
                    "org.kde.plasmashell",
                    "/PlasmaShell",
                    "org.kde.PlasmaShell.evaluateScript",
                    script,
                ])
                .output(),
            Self::Gdbus => Command::new("gdbus")
                .args([
                    "call",
                    "--session",
                    "--dest",
                    "org.kde.plasmashell",
                    "--object-path",
                    "/PlasmaShell",
                    "--method",
                    "org.kde.PlasmaShell.evaluateScript",
                    script,
                ])
                .output(),
            Self::Busctl => Command::new("busctl")
                .args([
                    "--user",
                    "call",
                    "org.kde.plasmashell",
                    "/PlasmaShell",
                    "org.kde.PlasmaShell",
                    "evaluateScript",
                    "s",
                    script,
                ])
                .output(),
        };

        match result {
            Ok(output) => BackendResult::Finished(output),
            Err(error) if error.kind() == io::ErrorKind::NotFound => BackendResult::NotInstalled,
            Err(error) => BackendResult::Finished(Output {
                status: failure_exit_status(),
                stdout: Vec::new(),
                stderr: error.to_string().into_bytes(),
            }),
        }
    }
}

#[cfg(unix)]
fn failure_exit_status() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(1 << 8)
}

#[cfg(not(unix))]
compile_error!("sticky only supports Linux/Unix systems with KDE Plasma");

fn parse_dbus_output(backend: &str, output: Output) -> Result<String, String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    if let Some(position) = combined.find("STICKY_OK:") {
        let value = &combined[position + "STICKY_OK:".len()..];
        let widget_id: String = value
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect();
        return Ok(if widget_id.is_empty() {
            "unknown".into()
        } else {
            widget_id
        });
    }

    if let Some(position) = combined.find("STICKY_ERROR:") {
        let message = &combined[position + "STICKY_ERROR:".len()..];
        let message = message
            .trim_matches(|ch: char| ch == '\'' || ch == '"' || ch == ')' || ch == ',' || ch.is_whitespace())
            .replace("\\n", "\n");
        return Err(format!("Plasma reported: {message}"));
    }

    if !output.status.success() {
        let details = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!("D-Bus call via {backend} failed: {details}"));
    }

    // PlasmaShell.evaluateScript has a void D-Bus return type. qdbus therefore
    // prints nothing on success, while gdbus commonly prints `()`.
    if stdout.trim().is_empty() || stdout.trim() == "()" {
        return Ok("unknown".into());
    }

    Err(format!(
        "unexpected response from {backend}: {}",
        stdout.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_plain_text_to_safe_html() {
        let html = plain_text_to_qt_html("A < B & C\n\t'quote'");
        assert_eq!(
            html,
            "<html><body><p>A &lt; B &amp; C<br/>&nbsp;&nbsp;&nbsp;&nbsp;&#39;quote&#39;</p></body></html>"
        );
    }

    #[test]
    fn generated_uuid_has_v4_shape() {
        let uuid = generate_uuid_v4().unwrap();
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[14..15], "4");
        assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"));
    }

    #[test]
    fn accepts_empty_successful_dbus_response() {
        use std::os::unix::process::ExitStatusExt;

        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };

        assert_eq!(parse_dbus_output("qdbus6", output), Ok("unknown".into()));
    }

    #[test]
    fn accepts_gdbus_void_response() {
        use std::os::unix::process::ExitStatusExt;

        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"()\n".to_vec(),
            stderr: Vec::new(),
        };

        assert_eq!(parse_dbus_output("gdbus", output), Ok("unknown".into()));
    }

    #[test]
    fn default_script_targets_first_panel() {
        let options = Options {
            text: "x".into(),
            panel_id: None,
            screen: None,
            color: "yellow".into(),
            pin_open: false,
        };
        let script = build_plasma_script("abc", &options);
        assert!(script.contains("panelIds"));
        assert!(script.contains("panel.addWidget('org.kde.plasma.notes')"));
        assert!(script.contains("widget.writeConfig('noteId', 'abc')"));
    }
}
