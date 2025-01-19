//! Module for parsing `.cwsh` files.

/// Possible errors that can occur while parsing a `.cwsh` file.
#[derive(Debug, Clone)]
enum ParseError {
    /// This line does not form a valid instruction.
    InvalidInstruction,
    // /// Expected a continuation line, but got something else.
    // ExpectedContinuation,
    /// Did not expect a continuation line, but got one.
    UnexpectedContinuation,
}

/// Represents a single line of instruction in a `.cwsh` file.
#[derive(Debug, PartialEq)]
enum Instruction {
    /// Persistent configuration instruction or metadata. (`@@`)
    PersistentConfig(String),
    /// Temporary configuration instruction. (`@`)
    TemporaryConfig(String),
    /// Print a string as it is. (`%`)
    Print(String),
    /// Marker. (`!`)
    Marker(String),
    /// Comment (`#`) or empty line.
    Empty,
    /// One-line shell command. (`$`)
    Command(String),
    /// Continuation of a multi-line shell command. (`>`)
    Continuation(String),
}

impl Instruction {
    /// Parse a line into an `Instruction`.
    fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();
        let Some(first) = s.chars().next() else {
            return Ok(Self::Empty);
        };
        let trimmed = s[1..].trim().to_string();
        match first {
            '@' => {
                let Some(second) = s.chars().nth(1) else {
                    return Err(ParseError::InvalidInstruction);
                };
                match second {
                    '@' => Ok(Self::PersistentConfig(s[2..].trim().to_string())),
                    _ => Ok(Self::TemporaryConfig(trimmed)),
                }
            },
            '%' => Ok(Self::Print(trimmed)),
            '!' => Ok(Self::Marker(trimmed)),
            '#' => Ok(Self::Empty),
            '$' => Ok(Self::Command(trimmed)),
            '>' => Ok(Self::Continuation(trimmed)),
            _ => Err(ParseError::InvalidInstruction),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_parse_with_space() {
        use Instruction::*;
        let instructions = [
            ("@@ persistent config", PersistentConfig("persistent config".to_string())),
            ("@ temporary config", TemporaryConfig("temporary config".to_string())),
            ("% print", Print("print".to_string())),
            ("! marker", Marker("marker".to_string())),
            ("# comment", Empty),
            ("$ command", Command("command".to_string())),
            ("> continuation", Continuation("continuation".to_string())),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&Instruction::parse(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_instruction_parse_without_space() {
        use Instruction::*;
        let instructions = [
            ("@@persistent config", PersistentConfig("persistent config".to_string())),
            ("@temporary config", TemporaryConfig("temporary config".to_string())),
            ("%print", Print("print".to_string())),
            ("!marker", Marker("marker".to_string())),
            ("#comment", Empty),
            ("$command", Command("command".to_string())),
            (">continuation", Continuation("continuation".to_string())),
        ];
        for (input, expected) in instructions.iter() {
            assert_eq!(&Instruction::parse(input).unwrap(), expected);
        }
    }

    #[test]
    fn test_instruction_parse_empty() {
        let empty_lines = [
            "",
            " ",
            "  ",
            "\t",
            "\t ",
            " \t",
            "\n",
        ];
        for line in empty_lines.iter() {
            assert_eq!(&Instruction::parse(line).unwrap(), &Instruction::Empty);
        }
    }
}
