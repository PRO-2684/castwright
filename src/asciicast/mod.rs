//! Module for modeling and streaming [asciicast v2](https://docs.asciinema.org/manual/asciicast/v2/) content.

mod event;
mod header;
use super::{util, ErrorType};
use event::Event;
use header::Header;
use serde_json::ser::to_writer;
use std::{collections::HashMap, io::Write};

/// An [asciicast v2](https://docs.asciinema.org/manual/asciicast/v2/) instance, streaming content to a writer.
///
/// ## Instantiation
///
/// Can be instantiated using the [`AsciiCast::new`] method, which accepts a mutable reference to a writer and returns an asciicast instance. The instance streams content to the writer as methods are called. See [Header](#header) and [Events](#events) section for more information.
///
/// ## Header
///
/// You can modify the header of the asciicast using the following methods:
///
/// - [`width`](AsciiCast::width): Set the initial terminal width.
/// - [`height`](AsciiCast::height): Set the initial terminal height.
/// - [`timestamp`](AsciiCast::timestamp): Set the unix timestamp of the beginning of the recording session in seconds.
/// - [`idle_time_limit`](AsciiCast::idle_time_limit): Set the idle time limit.
/// - [`title`](AsciiCast::title): Set the title of the asciicast.
/// - [`capture`](AsciiCast::capture): Set the captured environment variables.
///
/// After you've finished, you can write the header using the [`write_header`](AsciiCast::write_header) method explicitly. If you don't, the header will be written implicitly when you write the first event, or when the asciicast instance is dropped. Note that the header can only be written once, either explicitly or implicitly, or a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error will be returned.
///
/// ## Events
///
/// You can add events using the following methods:
///
/// - [`output`](AsciiCast::output): Write an output event.
/// - [`input`](AsciiCast::input): Write an input event.
/// - [`marker`](AsciiCast::marker): Write a marker event.
/// - [`resize`](AsciiCast::resize): Write a resize event.
///
/// ## Output
///
/// The asciicast will be streamed to the writer you provided, every time you add an event or write the header. When it is dropped, it will call [`finish`](AsciiCast::finish), which tries to:
///
/// - Write the header if it hasn't been written yet
/// - Flush the writer
///
/// Since it is not possible to return an error when dropping the asciicast instance, the error will be printed to `stderr` if it occurs. If you want to handle the error, you should call [`finish`](AsciiCast::finish) explicitly and handle the error yourself.
///
/// Due to the reason that `Drop` borrows `self` as mutable, you can't access the writer before the asciicast instance goes out of scope. Consider the following example:
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
pub struct AsciiCast<'a, T>
where
    T: Write + ?Sized, {
    header: Option<Header>,
    writer: &'a mut T,
}

impl<'a, T> AsciiCast<'a, T>
where
    T: Write + ?Sized, {
    /// Create a new asciicast.
    ///
    /// Alternatively, `AsciiCast` implements `From<&mut T> for AsciiCast<T> where T: Write`, so you can use `into()` to create an asciicast instance:
    ///
    /// ## Example
    ///
    /// ```rust
    /// use std::io::sink;
    /// use castwright::AsciiCast;
    /// let mut writer = sink();
    /// let mut asciicast = AsciiCast::new(&mut writer);
    /// ```
    ///
    /// ```rust
    /// use std::io::{Sink, sink};
    /// use castwright::AsciiCast;
    /// let mut writer = sink();
    /// let mut asciicast: AsciiCast<Sink> = (&mut writer).into();
    /// ```
    pub fn new(writer: &'a mut T) -> Self {
        Self {
            header: Some(Header::new()),
            writer,
        }
    }

    // Header
    /// Set the [initial terminal width](https://docs.asciinema.org/manual/asciicast/v2/#width).
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    pub fn width(&mut self, width: u16) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.width = width;
        Ok(self)
    }
    /// Set the [initial terminal height](https://docs.asciinema.org/manual/asciicast/v2/#height).
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    pub fn height(&mut self, height: u16) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.height = height;
        Ok(self)
    }
    /// Set the [unix timestamp of the beginning of the recording session](https://docs.asciinema.org/manual/asciicast/v2/#timestamp) in seconds.
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    pub fn timestamp(&mut self, timestamp: u64) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.timestamp = Some(timestamp);
        Ok(self)
    }
    /// Set the [idle time limit](https://docs.asciinema.org/manual/asciicast/v2/#idle_time_limit).
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    pub fn idle_time_limit(&mut self, idle_time_limit: f64) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.idle_time_limit = Some(idle_time_limit);
        Ok(self)
    }
    /// Set the [title of the asciicast](https://docs.asciinema.org/manual/asciicast/v2/#title).
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    pub fn title(&mut self, title: String) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.title = Some(title);
        Ok(self)
    }
    /// Set the [captured environment variables](https://docs.asciinema.org/manual/asciicast/v2/#env).
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    pub fn capture(&mut self, env_vars: HashMap<String, String>) -> Result<&mut Self, ErrorType> {
        self.get_header_mut()?.env = if env_vars.is_empty() {
            None
        } else {
            Some(env_vars)
        };
        Ok(self)
    }
    /// Write the header to the writer.
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written, or [`Json`](ErrorType::Json) error if there was an error while serializing the header, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    pub fn write_header(&mut self) -> Result<&mut Self, ErrorType> {
        let header = self.header.take().ok_or(ErrorType::HeaderAlreadyWritten)?;
        to_writer(&mut self.writer, &header)?;
        writeln!(&mut self.writer)?;
        Ok(self)
    }
    /// Try to write the header to the writer. Does nothing if the header has already been written.
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if there was an error while serializing the header, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    fn try_write_header(&mut self) -> Result<(), ErrorType> {
        if self.header.is_some() {
            self.write_header()?;
        }
        Ok(())
    }
    /// Get a mutable reference to the header.
    ///
    /// ## Errors
    ///
    /// Returns a [`HeaderAlreadyWritten`](ErrorType::HeaderAlreadyWritten) error if the header has already been written.
    fn get_header_mut(&mut self) -> Result<&mut Header, ErrorType> {
        self.header.as_mut().ok_or(ErrorType::HeaderAlreadyWritten)
    }

    // Events
    /// Write an [output event](https://docs.asciinema.org/manual/asciicast/v2/#o-output-data-written-to-a-terminal).
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if serialization fails, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    pub fn output(&mut self, time: u128, data: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.event(&Event::output(time, data))?;
        Ok(self)
    }
    /// Write an [input event](https://docs.asciinema.org/manual/asciicast/v2/#i-input-data-read-from-a-terminal).
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if serialization fails, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    pub fn input(&mut self, time: u128, data: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.event(&Event::input(time, data))?;
        Ok(self)
    }
    /// Write a [marker event](https://docs.asciinema.org/manual/asciicast/v2/#m-marker).
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if serialization fails, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    pub fn marker(&mut self, time: u128, name: &str) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.event(&Event::marker(time, name))?;
        Ok(self)
    }
    /// Write a [resize event](https://docs.asciinema.org/manual/asciicast/v2/#r-resize).
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if serialization fails, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    pub fn resize(&mut self, time: u128, columns: u16, rows: u16) -> Result<&mut Self, ErrorType> {
        self.try_write_header()?;
        self.event(&Event::resize(time, &format!("{columns}x{rows}")))?;
        Ok(self)
    }
    /// Write an event to the writer.
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if serialization fails, or an [`Io`](ErrorType::Io) error if writing to the writer fails.
    fn event(&mut self, event: &Event) -> Result<(), ErrorType> {
        event.write(&mut self.writer)?;
        writeln!(&mut self.writer)?;
        Ok(())
    }

    // Finish
    /// Finish writing the asciicast. To be specific:
    ///
    /// - Writes the header if it hasn't been written yet
    /// - Flushes the writer
    ///
    /// ## Errors
    ///
    /// Returns a [`Json`](ErrorType::Json) error if there was an error while serializing the header, or an [`Io`](ErrorType::Io) error if writing or flushing fails.
    pub fn finish(&mut self) -> Result<(), ErrorType> {
        self.try_write_header()?;
        self.writer.flush()?;
        Ok(())
    }
}

impl<'a, T> From<&'a mut T> for AsciiCast<'a, T>
where
    T: Write + ?Sized,
{
    fn from(writer: &'a mut T) -> Self {
        Self::new(writer)
    }
}

impl<T> Drop for AsciiCast<'_, T>
where
    T: Write + ?Sized, {
    fn drop(&mut self) {
        if let Err(err) = self.finish() {
            eprintln!("Error while calling `finish` on AsciiCast: {err}");
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
        let mut writer = std::io::sink();
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
        let mut writer = std::io::sink();
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
