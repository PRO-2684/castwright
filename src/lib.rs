mod instruction;
pub use instruction::{ConfigInstruction, Instruction};

/// Possible errors that can occur while parsing a `.cwsh` file.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    // General parsing errors
    /// Unknown instruction: The first character of the line is not recognized.
    UnknownInstruction,
    /// Malformed instruction: The instruction is not in the expected format.
    MalformedInstruction,
    /// Did not expect a continuation line, but got one.
    UnexpectedContinuation,
}
