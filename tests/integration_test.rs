use castwright::{CastWright, Error, ErrorType};
use std::{fs::File, io::Read};

const INPUT_DIR: &str = "tests/input/";
const OUTPUT_DIR: &str = "tests/output/";

/// A test case, contains the name of the test, the input file and the expected output file.
struct TestCase {
    name: String,
    input: File,
    output: File,
}

/// Read `tests/input/<name>.cwrt` and `tests/output/<name>.cast` files, return a iterator of `TestCase`.
fn test_cases() -> impl Iterator<Item = TestCase> {
    let input_dir = std::fs::read_dir(INPUT_DIR).unwrap();
    input_dir.map(|entry| {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        let name = name.trim_end_matches(".cwrt").to_string();
        let input = File::open(format!("{INPUT_DIR}{name}.cwrt")).unwrap();
        let output = File::open(format!("{OUTPUT_DIR}{name}.cast")).unwrap();
        TestCase {
            name,
            input,
            output,
        }
    })
}

#[test]
fn test() -> Result<(), Error> {
    let castwright = CastWright::new();
    for mut case in test_cases() {
        let mut reader = std::io::BufReader::new(case.input);
        let mut writer = Vec::new();
        castwright.run(&mut reader, &mut writer)?;

        let mut expected = Vec::new();
        case.output
            .read_to_end(&mut expected)
            .map_err(|err| ErrorType::Io(err).with_line(0))?;

        let output = String::from_utf8(writer).unwrap();
        let expected = String::from_utf8(expected).unwrap();

        // Compare output line by line (to avoid differences in line endings)
        for (i, (output_line, expected_line)) in output.lines().zip(expected.lines()).enumerate() {
            assert_eq!(
                output_line,
                expected_line,
                "Test case: {}, line: {}",
                case.name,
                i + 1
            );
        }
    }
    Ok(())
}
