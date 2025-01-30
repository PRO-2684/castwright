use argh::FromArgs;
use castwright::{CastWright, Error, ErrorType};
use disperror::DispError;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// 🎥 Scripted terminal recording.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help"))]
struct Args {
    /// the path to the input file (CastWright script `.cwrt`), or stdin if not provided
    #[argh(option, short = 'i')]
    input: Option<String>,
    /// the path to the output file (asciicast `.cast`), or stdout if not provided; If provided, preview mode will be enabled
    #[argh(option, short = 'o')]
    output: Option<String>,
    /// execute and capture the output of shell commands
    #[argh(switch, short = 'x')]
    execute: bool,
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

fn main() -> Result<(), DispError<Error>> {
    let args: Args = argh::from_env();

    let reader: &mut dyn Read = match &args.input {
        Some(path) => &mut file(path, false)?,
        None => &mut std::io::stdin().lock(),
    };
    let mut reader = std::io::BufReader::new(reader);

    let mut writer: &mut dyn Write = match &args.output {
        Some(path) => &mut file(path, true)?,
        None => &mut std::io::stdout().lock(),
    };

    CastWright::new()
        .execute(args.execute)
        .preview(args.output.is_some())
        .run(&mut reader, &mut writer)?;

    Ok(())
}
