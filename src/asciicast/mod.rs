//! Module for serializing to [asciicast v2 format](https://docs.asciinema.org/manual/asciicast/v2/).

mod event;
mod header;
use super::{Error, ErrorType};
use event::Event;
use header::Header;
use std::io::Write;

/// An asciicast v2 file. Usually you'll only need the [`write`](AsciiCast::write) method.
///
/// ## Creation
///
/// Can be created using the [`AsciiCast::new`] method, which returns an empty asciicast without any events. To create an asciicast with events, you can [`parse`](crate::Script::parse) from a reader, or call respective methods to add events (see "Events" section).
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
/// You can write the asciicast to a writer (`impl std::io::Write`) using the [`write`](AsciiCast::write) method.
pub struct AsciiCast<'a> {
    header: Header,
    events: Vec<Event>,
    writer: &'a mut dyn Write,
}

impl<'a> AsciiCast<'a> {
    /// Create a new asciicast.
    pub fn new(writer: &'a mut dyn Write) -> Self {
        Self {
            header: Header::new(),
            events: Vec::new(),
            writer,
        }
    }

    // Header
    /// Set the initial terminal width.
    pub fn width(&mut self, width: u16) -> &mut Self {
        self.header.width = width;
        self
    }
    /// Set the initial terminal height.
    pub fn height(&mut self, height: u16) -> &mut Self {
        self.header.height = height;
        self
    }
    /// Set the unix timestamp of the beginning of the recording session.
    pub fn timestamp(&mut self, timestamp: u64) -> &mut Self {
        self.header.timestamp = Some(timestamp);
        self
    }
    /// Set the idle time limit.
    pub fn idle_time_limit(&mut self, idle_time_limit: f64) -> &mut Self {
        self.header.idle_time_limit = Some(idle_time_limit);
        self
    }
    /// Set the title of the asciicast.
    pub fn title(&mut self, title: String) -> &mut Self {
        self.header.title = Some(title);
        self
    }

    // Events
    /// Add an output event to the asciicast.
    pub fn output(&mut self, time: u64, data: String) -> &mut Self {
        self.events.push(Event::output(time, data));
        self
    }
    /// Add an input event to the asciicast.
    pub fn input(&mut self, time: u64, data: String) -> &mut Self {
        self.events.push(Event::input(time, data));
        self
    }
    /// Add a marker event to the asciicast.
    pub fn marker(&mut self, time: u64, name: String) -> &mut Self {
        self.events.push(Event::marker(time, name));
        self
    }
    /// Add a resize event to the asciicast.
    pub fn resize(&mut self, time: u64, columns: u16, rows: u16) -> &mut Self {
        self.events.push(Event::resize(time, columns, rows));
        self
    }

    // Output
    /// Write the asciicast to a writer.
    pub fn write(&mut self) -> Result<(), Error> {
        use serde_json::ser::to_writer;
        to_writer(&mut self.writer, &self.header).map_err(|err| ErrorType::Json(err).with_line(0))?;
        for event in &self.events {
            writeln!(&mut self.writer).map_err(|err| ErrorType::Io(err).with_line(0))?;
            to_writer(&mut self.writer, event).map_err(|err| ErrorType::Json(err).with_line(0))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let mut writer = Vec::new();
        let mut asciicast = AsciiCast::new(&mut writer);
        asciicast
            .width(80)
            .height(24)
            .timestamp(1_000_000)
            .idle_time_limit(2.5)
            .title("Test".to_string())
            .output(0, "Hello, world!".to_string())
            .input(100, "echo Hello, world!".to_string())
            .marker(200, "marker".to_string())
            .resize(300, 80, 25);
        asciicast.write().unwrap();
        let expected = r#"{"version":2,"width":80,"height":24,"timestamp":1000000,"idle_time_limit":2.5,"title":"Test"}
[0.0,"o","Hello, world!"]
[0.0001,"i","echo Hello, world!"]
[0.0002,"m","marker"]
[0.0003,"r","80x25"]"#;
        assert_eq!(String::from_utf8(writer).unwrap(), expected);
    }
}
