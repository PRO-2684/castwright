use castwright::Script;

fn main() {
    let file = std::fs::File::open("demo.cw").unwrap();
    let reader = std::io::BufReader::new(file);
    match Script::parse(reader) {
        Ok(script) => script.execute(),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
