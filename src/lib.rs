mod instruction;
pub use instruction::{ConfigInstruction, Instruction, Script};

/// Possible errors that can occur while parsing a `.cw` file.
#[derive(Debug)]
pub enum ParseError {
    // General parsing errors
    /// An io error occurred while reading the file.
    Io(std::io::Error),
    /// Unknown instruction: The first character of the line is not recognized.
    UnknownInstruction,
    /// Malformed instruction: The instruction is not in the expected format.
    MalformedInstruction,
    /// Did not expect a continuation line, but got one.
    UnexpectedContinuation,
}
