use castwright::{ParseError, Script};
use disperror::DispError;

fn main() -> Result<(), DispError<ParseError>> {
    let file = std::fs::File::open("demo.cw").unwrap();
    let reader = std::io::BufReader::new(file);
    let script = Script::parse(reader)?;
    let result = script.execute();
    println!("{:#?}", result);
    Ok(())
}
