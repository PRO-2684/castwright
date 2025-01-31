//! Module for imitating the `cd` shell command.

use super::{BuiltInCommand, ErrorType, ExecutionContext};

pub struct Cd {
    /// Directory to change to.
    directory: String,
}

impl BuiltInCommand for Cd {
    fn new(arg: &str) -> Self {
        Self {
            directory: arg.to_string(),
        }
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<(), ErrorType> {
        let path = context.directory.join(&self.directory).canonicalize()?;
        // Ensure the path exists and is a directory
        if !path.exists() {
            return Err(ErrorType::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No such file or directory: {}", self.directory),
            )));
        }
        if !path.is_dir() {
            return Err(ErrorType::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Not a directory: {}", self.directory),
            )));
        }
        // Change the directory
        context.directory = path;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn cd_relative() {
        let current = PathBuf::from(".").canonicalize().unwrap();
        let parent = current.parent().unwrap().to_path_buf();

        let cd = Cd::new("..");
        let mut context = ExecutionContext::new();
        cd.execute(&mut context).unwrap();

        assert_eq!(context.directory, parent);
    }

    #[test]
    fn cd_absolute() {
        let current = PathBuf::from(".").canonicalize().unwrap();
        let parent = current.parent().unwrap().to_path_buf();

        let cd = Cd::new(parent.to_str().unwrap());
        let mut context = ExecutionContext::new();
        cd.execute(&mut context).unwrap();

        assert_eq!(context.directory, parent);
    }
}
