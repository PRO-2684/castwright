use castwright::Script;

fn main() {
    let file = std::fs::File::open("demo.cw").unwrap();
    let reader = std::io::BufReader::new(file);
    let script = Script::parse(reader).unwrap();
    script.execute();
}
