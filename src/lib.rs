mod instruction;
pub use instruction::{ConfigInstruction, Instruction, Script};

/// Possible errors that can occur while parsing a `.cw` file.
#[derive(Debug)]
pub enum ParseError {
    // General parsing errors
    /// An io error occurred while reading the file.
    Io(std::io::Error),
    /// Unknown instruction: The first character of the line is not recognized. The contained `usize` is the line number, starting at 1.
    UnknownInstruction(usize),
    /// Malformed instruction: The instruction is not in the expected format. The contained `usize` is the line number, starting at 1.
    MalformedInstruction(usize),
    /// Did not expect a continuation line, but got one. The contained `usize` is the line number, starting at 1.
    UnexpectedContinuation(usize),
}

impl ParseError {
    pub fn with_line(self, line: usize) -> Self {
        match self {
            ParseError::UnknownInstruction(_) => ParseError::UnknownInstruction(line),
            ParseError::MalformedInstruction(_) => ParseError::MalformedInstruction(line),
            ParseError::UnexpectedContinuation(_) => ParseError::UnexpectedContinuation(line),
            other => other,
        }
    }
}
