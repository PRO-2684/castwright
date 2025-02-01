use castwright::{CastWright, Error, ErrorType};
use std::{fs::File, io::{Read, BufReader}};

const INPUT_DIR: &str = "tests/input/";
const OUTPUT_DIR: &str = "tests/output/";
const SUCCESS_DIR: &str = "tests/success/";
const FAILURE_DIR: &str = "tests/failure/";

// Input-Output tests, without execution.

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
fn input_output_tests() -> Result<(), Error> {
    let castwright = CastWright::new();
    for mut case in test_cases() {
        let mut reader = BufReader::new(case.input);
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

// Success or failure tests, with execution. Linux only.

/// Read all files in a directory, return a iterator of `BufReader<File>`.
#[cfg(target_os = "linux")]
fn test_files(dir: &str) -> impl Iterator<Item = (String, BufReader<File>)> {
    let dir = std::fs::read_dir(dir).unwrap();
    dir.map(|entry| {
        let entry = entry.unwrap();
        let file = File::open(entry.path()).unwrap();
        (entry.file_name().into_string().unwrap(), BufReader::new(file))
    })
}

#[cfg(target_os = "linux")]
#[test]
fn success_tests() {
    let mut writer = std::io::sink(); // Discard output
    for (name, mut reader) in test_files(SUCCESS_DIR) {
        let castwright = CastWright::new().execute(true);
        castwright.run(&mut reader, &mut writer).expect(&format!("Test case `{}` should succeed", name));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn failure_tests() {
    let mut writer = std::io::sink(); // Discard output
    for (name, mut reader) in test_files(FAILURE_DIR) {
        let castwright = CastWright::new().execute(true);
        castwright.run(&mut reader, &mut writer).expect_err(&format!("Test case `{}` should fail", name));
    }
}
