//! Module for serializing to [asciicast v2 format](https://docs.asciinema.org/manual/asciicast/v2/).

mod event;
mod header;
use super::ErrorType;
use crate::util::capture_env_vars;
use event::Event;
use header::Header;
use serde_json::ser::to_writer;
use std::io::Write;

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
/// After you've finished, write the header to the asciicast using the [`write_header`](AsciiCast::write_header) method explicitly. If you don't, the header will be written implicitly when you write the first event. Note that the header can only be written once, either explicitly or implicitly.
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
/// The asciicast will be streamed to the writer you provided, every time you add an event or write the header. You can finish writing the asciicast using the [`finish`](AsciiCast::finish) method, which consumes the asciicast and flushes the writer.
pub struct AsciiCast<'a> {
    header: Header,
    writer: &'a mut dyn Write,
    header_written: bool,
}

impl<'a> AsciiCast<'a> {
    /// Create a new asciicast.
    pub fn new(writer: &'a mut dyn Write) -> Self {
        Self {
            header: Header::new(),
            writer,
            header_written: false,
        }
    }

    // Header
    /// Set the initial terminal width.
    pub fn width(&mut self, width: u16) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        self.header.width = width;
        Ok(self)
    }
    /// Set the initial terminal height.
    pub fn height(&mut self, height: u16) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        self.header.height = height;
        Ok(self)
    }
    /// Set the unix timestamp of the beginning of the recording session.
    pub fn timestamp(&mut self, timestamp: u64) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        self.header.timestamp = Some(timestamp);
        Ok(self)
    }
    /// Set the idle time limit.
    pub fn idle_time_limit(&mut self, idle_time_limit: f64) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        self.header.idle_time_limit = Some(idle_time_limit);
        Ok(self)
    }
    /// Set the title of the asciicast.
    pub fn title(&mut self, title: String) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        self.header.title = Some(title);
        Ok(self)
    }
    /// Set the captured environment variables.
    pub fn capture(&mut self, env_vars: Vec<String>) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        self.header.env = capture_env_vars(env_vars);
        Ok(self)
    }
    /// Write the header to the writer.
    pub fn write_header(&mut self) -> Result<&mut Self, ErrorType> {
        self.assert_header_not_written()?;
        to_writer(&mut self.writer, &self.header)?;
        writeln!(&mut self.writer)?;
        self.header_written = true;
        Ok(self)
    }
    /// Try to write the header to the writer. Does nothing if the header has already been written.
    fn try_write_header(&mut self) -> Result<(), ErrorType> {
        if !self.header_written {
            self.write_header()?;
        }
        Ok(())
    }
    /// Errors if the header has already been written.
    fn assert_header_not_written(&self) -> Result<(), ErrorType> {
        if self.header_written {
            Err(ErrorType::HeaderAlreadyWritten)
        } else {
            Ok(())
        }
    }

    // Events
    /// Write an output event to the asciicast.
    pub fn output(&mut self, time: u64, data: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::output(time, data))?;
        Ok(self)
    }
    /// Write an input event to the asciicast.
    pub fn input(&mut self, time: u64, data: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::input(time, data))?;
        Ok(self)
    }
    /// Write a marker event to the asciicast.
    pub fn marker(&mut self, time: u64, name: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.write_event(&Event::marker(time, name))?;
        Ok(self)
    }
    /// Write a resize event to the asciicast.
    pub fn resize(&mut self, time: u64, columns: u16, rows: u16) -> Result<&mut Self, ErrorType> {
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
    /// Finish writing the asciicast, consuming self.
    pub fn finish(mut self) -> Result<(), ErrorType> {
        self.try_write_header()?;
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_write_header() -> Result<(), ErrorType> {
        let mut writer = Vec::new();
        let mut asciicast = AsciiCast::new(&mut writer);
        asciicast
            .width(80)?
            .height(24)?
            .timestamp(1_000_000)?
            .idle_time_limit(2.5)?
            .title("Test".to_string())?
            .capture(vec![])?
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
        let mut asciicast = AsciiCast::new(&mut writer);
        asciicast
            .width(80)?
            .height(24)?
            .timestamp(1_000_000)?
            .idle_time_limit(2.5)?
            .title("Test".to_string())?
            .capture(vec![])?
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
