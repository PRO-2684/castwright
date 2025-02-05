//! Module for serializing to [asciicast v2 format](https://docs.asciinema.org/manual/asciicast/v2/).

mod event;
mod header;
use super::ErrorType;
use event::Event;
use header::Header;
use serde_json::ser::to_writer;
use std::{collections::HashMap, io::Write};

/// An asciicast v2 file.
///
/// ## Creation
///
/// Can be created using the [`AsciiCast::new`] method, which accepts a writer and returns an empty asciicast without any events. To modify header and write events to an asciicast, you can call respective methods (see "Header" and "Events" section).
///
/// ## Modification
///
/// ### Header
///
/// You can modify the header of the asciicast using the following methods:
///
/// - [`width`](AsciiCast::width): Set the initial terminal width.
/// - [`height`](AsciiCast::height): Set the initial terminal height.
/// - [`timestamp`](AsciiCast::timestamp): Set the unix timestamp of the beginning of the recording session.
/// - [`idle_time_limit`](AsciiCast::idle_time_limit): Set the idle time limit.
/// - [`title`](AsciiCast::title): Set the title of the asciicast.
/// - [`capture`](AsciiCast::capture): Set the captured environment variables.
///
/// After you've finished, you can write the header to the asciicast using the [`write_header`](AsciiCast::write_header) method explicitly. If you don't, the header will be written implicitly when you write the first event, or when the asciicast is dropped. Note that the header can only be written once, either explicitly or implicitly, or a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error will be returned.
///
/// ### Events
///
/// You can add events to the asciicast using the following methods:
///
/// - [`output`](AsciiCast::output): Add an output event to the asciicast.
/// - [`input`](AsciiCast::input): Add an input event to the asciicast.
/// - [`marker`](AsciiCast::marker): Add a marker event to the asciicast.
/// - [`resize`](AsciiCast::resize): Add a resize event to the asciicast.
///
/// ## Output
///
/// The asciicast will be streamed to the writer you provided, every time you add an event or write the header. When it is dropped, it will call [`finish`](AsciiCast::finish), which tries to:
///
/// - Write the header if it hasn't been written yet
/// - Flush the writer
///
/// Since it is not possible to return an error when dropping the asciicast, the error will be printed to `stderr` if it occurs. If you want to handle the error, you should call [`finish`](AsciiCast::finish) explicitly and handle the error yourself.
///
/// Due to the reason that `Drop` borrows `self` as mutable, you can't access the writer before the asciicast goes out of scope. Consider the following example:
///
/// ```rust compile_fail
/// use castwright::AsciiCast;
/// let mut writer = Vec::new();
///
/// let mut asciicast = AsciiCast::new(&mut writer);
/// asciicast.output(0, "Hello, world!").unwrap();
///
/// let content = String::from_utf8_lossy(&writer); // error[E0502]: cannot borrow `writer` as immutable because it is also borrowed as mutable
/// let second_line = content.lines().nth(1).unwrap();
/// assert_eq!(second_line, r#"[0.000000,"o","Hello, world!"]"#);
/// ```
///
/// There are several workarounds to this problem:
///
/// ### Workaround 1 - No `let` binding (one-liner)
///
/// ```rust
/// # use castwright::AsciiCast;
/// # let mut writer = Vec::new();
/// // ...
/// AsciiCast::new(&mut writer).output(0, "Hello, world!").unwrap();
/// // ...
/// # let content = String::from_utf8_lossy(&writer);
/// # let second_line = content.lines().nth(1).unwrap();
/// # assert_eq!(second_line, r#"[0.000000,"o","Hello, world!"]"#);
/// ```
///
/// ### Workaround 2 - Scoping
///
/// ```rust
/// # use castwright::AsciiCast;
/// # let mut writer = Vec::new();
/// // ...
/// {
///     let mut asciicast = AsciiCast::new(&mut writer);
///     asciicast.output(0, "Hello, world!").unwrap();
/// } // <- `asciicast` goes out of scope and is dropped here
/// // ...
/// # let content = String::from_utf8_lossy(&writer);
/// # let second_line = content.lines().nth(1).unwrap();
/// # assert_eq!(second_line, r#"[0.000000,"o","Hello, world!"]"#);
/// ```
///
/// ### Workaround 3 - Explicitly drop
///
/// ```rust
/// # use castwright::AsciiCast;
/// # let mut writer = Vec::new();
/// // ...
/// let mut asciicast = AsciiCast::new(&mut writer);
/// asciicast.output(0, "Hello, world!").unwrap();
/// drop(asciicast); // <- Explicitly drop `asciicast`
/// // ...
/// # let content = String::from_utf8_lossy(&writer);
/// # let second_line = content.lines().nth(1).unwrap();
/// # assert_eq!(second_line, r#"[0.000000,"o","Hello, world!"]"#);
/// ```
pub struct AsciiCast<'a> {
    header: Option<Header>,
    writer: &'a mut dyn Write,
}

impl<'a> AsciiCast<'a> {
    /// Create a new asciicast.
    pub fn new(writer: &'a mut dyn Write) -> Self {
        Self {
            header: Some(Header::new()),
            writer,
        }
    }

    // Header
    /// Set the initial terminal width.
    pub fn width(&mut self, width: u16) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.width = width;
        Ok(self)
    }
    /// Set the initial terminal height.
    pub fn height(&mut self, height: u16) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.height = height;
        Ok(self)
    }
    /// Set the unix timestamp of the beginning of the recording session.
    pub fn timestamp(&mut self, timestamp: u64) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.timestamp = Some(timestamp);
        Ok(self)
    }
    /// Set the idle time limit.
    pub fn idle_time_limit(&mut self, idle_time_limit: f64) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.idle_time_limit = Some(idle_time_limit);
        Ok(self)
    }
    /// Set the title of the asciicast.
    pub fn title(&mut self, title: String) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.title = Some(title);
        Ok(self)
    }
    /// Set the captured environment variables.
    pub fn capture(&mut self, env_vars: HashMap<String, String>) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.env = if env_vars.is_empty() {
            None
        } else {
            Some(env_vars)
        };
        Ok(self)
    }
    /// Write the header to the writer.
    pub fn write_header(&mut self) -> Result<&mut Self, ErrorType> {
        let header = self.header.take().ok_or(ErrorType::HeaderAlreadyWritten)?;
        to_writer(&mut self.writer, &header)?;
        writeln!(&mut self.writer)?;
        Ok(self)
    }
    /// Try to write the header to the writer. Does nothing if the header has already been written.
    fn try_write_header(&mut self) -> Result<(), ErrorType> {
        if self.header.is_some() {
            self.write_header()?;
        }
        Ok(())
    }
    /// Get a mutable reference to the header, errors if the header has already been written.
    fn get_header_mut(&mut self) -> Result<&mut Header, ErrorType> {
        if let Some(header) = &mut self.header {
            Ok(header)
        } else {
            Err(ErrorType::HeaderAlreadyWritten)
        }
    }

    // Events
    /// Write an output event to the asciicast.
    pub fn output(&mut self, time: u128, data: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::output(time, data))?;
        Ok(self)
    }
    /// Write an input event to the asciicast.
    pub fn input(&mut self, time: u128, data: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::input(time, data))?;
        Ok(self)
    }
    /// Write a marker event to the asciicast.
    pub fn marker(&mut self, time: u128, name: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::marker(time, name))?;
        Ok(self)
    }
    /// Write a resize event to the asciicast.
    pub fn resize(&mut self, time: u128, columns: u16, rows: u16) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::resize(time, &format!("{}x{}", columns, rows)))?;
        Ok(self)
    }
    /// Write an event to the writer.
    fn write_event(&mut self, event: &Event) -> Result<(), ErrorType> {
        event.write(&mut self.writer)?;
        writeln!(&mut self.writer)?;
        Ok(())
    }

    // Finish
    /// Finish writing the asciicast. To be specific:
    ///
    /// - Writes the header if it hasn't been written yet
    /// - Flushes the writer
    pub fn finish(&mut self) -> Result<(), ErrorType> {
        self.try_write_header()?;
        self.writer.flush()?;
        Ok(())
    }
}

impl Drop for AsciiCast<'_> {
    fn drop(&mut self) {
        if let Err(err) = self.finish() {
            eprintln!("Error while calling `finish` on AsciiCast: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_write_header() -> Result<(), ErrorType> {
        let mut writer = Vec::new();

        AsciiCast::new(&mut writer)
            .width(80)?
            .height(24)?
            .timestamp(1_000_000)?
            .idle_time_limit(2.5)?
            .title("Test".to_string())?
            .capture(HashMap::new())?
            .write_header()?
            .output(0, "Hello, world!")?
            .input(100, "echo Hello, world!")?
            .marker(200, "marker")?
            .resize(300, 80, 25)?;

        let expected = r#"{"version":2,"width":80,"height":24,"timestamp":1000000,"idle_time_limit":2.5,"title":"Test"}
[0.000000,"o","Hello, world!"]
[0.000100,"i","echo Hello, world!"]
[0.000200,"m","marker"]
[0.000300,"r","80x25"]
"#;
        assert_eq!(String::from_utf8(writer).unwrap(), expected);
        Ok(())
    }

    #[test]
    fn implicit_write_header() -> Result<(), ErrorType> {
        let mut writer = Vec::new();

        AsciiCast::new(&mut writer)
            .width(80)?
            .height(24)?
            .timestamp(1_000_000)?
            .idle_time_limit(2.5)?
            .title("Test".to_string())?
            .capture(HashMap::new())?
            .output(0, "Hello, world!")?
            .input(100, "echo Hello, world!")?
            .marker(200, "marker")?
            .resize(300, 80, 25)?;

        let expected = r#"{"version":2,"width":80,"height":24,"timestamp":1000000,"idle_time_limit":2.5,"title":"Test"}
[0.000000,"o","Hello, world!"]
[0.000100,"i","echo Hello, world!"]
[0.000200,"m","marker"]
[0.000300,"r","80x25"]
"#;
        assert_eq!(String::from_utf8(writer).unwrap(), expected);
        Ok(())
    }

    #[test]
    fn explicit_header_already_written() -> Result<(), ErrorType> {
        let mut writer = Vec::new();
        let mut asciicast = AsciiCast::new(&mut writer);
        asciicast.width(80)?;
        asciicast.write_header()?;
        match asciicast.width(80) {
            Ok(_) => panic!("Expected error"),
            Err(err) => assert_eq!(err, ErrorType::HeaderAlreadyWritten),
        };
        Ok(())
    }

    #[test]
    fn implicit_header_already_written() -> Result<(), ErrorType> {
        let mut writer = Vec::new();
        let mut asciicast = AsciiCast::new(&mut writer);
        asciicast.width(80)?;
        asciicast.output(0, "Hello, world!")?;
        match asciicast.width(80) {
            Ok(_) => panic!("Expected error"),
            Err(err) => assert_eq!(err, ErrorType::HeaderAlreadyWritten),
        };
        Ok(())
    }
}
