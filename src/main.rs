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
    /// the path to the output file (asciicast `.cast`), or stdout if not provided
    #[argh(option, short = 'o')]
    output: Option<String>,
    /// execute and capture the output of shell commands
    #[argh(switch, short = 'x')]
    execute: bool,
}

fn main() -> Result<(), DispError<Error>> {
    let args: Args = argh::from_env();

    let reader: &mut dyn Read = match &args.input {
        Some(path) => {
            let path = Path::new(&path);
            &mut File::open(path).map_err(|err| ErrorType::Io(err).with_line(0))?
        }
        None => {
            let stdin = std::io::stdin();
            &mut stdin.lock()
        }
    };
    let mut reader = std::io::BufReader::new(reader);

    let mut writer: &mut dyn Write = match &args.output {
        Some(path) => {
            let path = Path::new(&path);
            &mut File::create(path).map_err(|err| ErrorType::Io(err).with_line(0))?
        }
        None => {
            let stdout = std::io::stdout();
            &mut stdout.lock()
        }
    };

    CastWright::new()
        .execute(args.execute)
        .run(&mut reader, &mut writer)?;

    Ok(())
}
