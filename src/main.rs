use argh::FromArgs;
use castwright::{CastWright, Error, ErrorType, VERSION};
use disperror::DispError;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

/// 🎥 Scripted terminal recording.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help"))]
struct Args {
    // Input and output
    /// the path to the input file (CastWright script `.cwrt`), or stdin if not provided
    #[argh(option, short = 'i')]
    input: Option<String>,
    /// the path to the output file (asciicast `.cast`), or stdout if not provided; If provided, preview mode will be enabled
    #[argh(option, short = 'o')]
    output: Option<String>,

    // Options
    /// execute and capture the output of shell commands
    #[argh(switch, short = 'x')]
    execute: bool,
    /// include timestamp information in the output
    #[argh(switch, short = 't')]
    timestamp: bool,

    // Help
    /// show version information and exit
    #[argh(switch, short = 'v')]
    version: bool,
}

/// Create or open a file at the given path.
fn file(path: &str, create: bool) -> Result<File, Error> {
    let path = Path::new(path);
    let file = if create {
        File::create(path)
    } else {
        File::open(path)
    };
    file.map_err(|e| ErrorType::Io(e).with_line(0))
}

/// Display a link in the terminal.
fn link(text: &str, url: &str) {
    print!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text);
}

/// Show version information.
fn version() {
    let github_base = "https://github.com/PRO-2684/castwright";
    println!("🎥 CastWright v{}", VERSION);
    link("GitHub", github_base);
    print!(" | ");
    link("Releases", &format!("{github_base}/releases"));
    print!(" | ");
    link("Docs.rs", "https://docs.rs/castwright");
    print!(" | ");
    link("Crates.io", "https://crates.io/crates/castwright");
    println!();
}

fn main() -> Result<(), DispError<Error>> {
    let args: Args = argh::from_env();

    if args.version {
        version();
        return Ok(());
    }

    let mut reader: &mut dyn BufRead = match &args.input {
        Some(path) => &mut BufReader::new(file(path, false)?),
        None => &mut std::io::stdin().lock(),
    };

    let mut writer: &mut dyn Write = match &args.output {
        Some(path) => &mut file(path, true)?,
        None => &mut std::io::stdout().lock(),
    };

    CastWright::new()
        .execute(args.execute)
        .timestamp(args.timestamp)
        .preview(args.output.is_some())
        .run(&mut reader, &mut writer)?;

    Ok(())
}
