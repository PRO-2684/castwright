use castwright::{ParseError, ParseErrorType, Script};
use disperror::DispError;
use std::fs::File;

fn main() -> Result<(), DispError<ParseError>> {
    let input = File::open("demo.cw").unwrap();
    let reader = std::io::BufReader::new(input);
    let script = Script::parse(reader)?;
    let cast = script.execute();
    let mut output = File::create("demo.cast").unwrap();
    cast.write(&mut output)
        .map_err(|err| ParseErrorType::Json(err).with_line(0))?;
    Ok(())
}
