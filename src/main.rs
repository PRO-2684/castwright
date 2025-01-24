use argh::FromArgs;
use castwright::{Error, ErrorType, Script};
use disperror::DispError;
use std::{
    fs::File,
    io::{Read, Write},
};

/// 🎥 Scripted terminal recording.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help"))]
struct Args {
    /// the path to the input file (castwright script `.cw`), or stdin if not provided
    #[argh(option, short = 'i')]
    input: Option<String>,
    /// the path to the output file (asciicast `.cast`), or stdout if not provided
    #[argh(option, short = 'o')]
    output: Option<String>,
    /// execute and capture the output of shell commands, instead of using dummy output (not implemented)
    #[argh(switch, short = 'x')]
    execute: bool,
}

/// Get a reader for the input, either from a file or stdin.
fn get_reader(input: &Option<String>) -> Result<Box<dyn Read>, Error> {
    match input {
        Some(path) => {
            let path = std::path::Path::new(&path);
            File::open(path)
                .map(|f| Box::new(f) as Box<dyn Read>)
                .map_err(|err| ErrorType::Io(err).with_line(0))
        }
        None => {
            let stdin = std::io::stdin();
            Ok(Box::new(stdin.lock()))
        }
    }
}

/// Get a writer for the output, either from a file or stdout.
fn get_writer(output: &Option<String>) -> Result<Box<dyn Write>, Error> {
    match output {
        Some(path) => {
            let path = std::path::Path::new(&path);
            File::create(path)
                .map(|f| Box::new(f) as Box<dyn Write>)
                .map_err(|err| ErrorType::Io(err).with_line(0))
        }
        None => {
            let stdout = std::io::stdout();
            Ok(Box::new(stdout.lock()))
        }
    }
}

fn main() -> Result<(), DispError<Error>> {
    let args: Args = argh::from_env();
    if args.execute {
        return Err(ErrorType::NotImplemented("`--execute` flag")
            .with_line(0)
            .into());
    }
    let input = get_reader(&args.input)?;
    let reader = std::io::BufReader::new(input);
    let script = Script::parse(reader)?;
    let cast = script.execute();
    let mut output = get_writer(&args.output)?;
    cast.write(&mut output)?;
    Ok(())
}
