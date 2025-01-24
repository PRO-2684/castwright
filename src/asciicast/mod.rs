//! Module for serializing to [asciicast v2 format](https://docs.asciinema.org/manual/asciicast/v2/).

mod event;
mod header;
use event::Event;
use header::Header;

/// An asciicast v2 file.
#[derive(Debug)]
pub struct AsciiCast {
    header: Header,
    events: Vec<Event>,
}

impl AsciiCast {
    /// Create a new asciicast.
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            events: Vec::new(),
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
    pub(crate) fn output(&mut self, time: u64, data: String) -> &mut Self {
        self.events.push(Event::output(time, data));
        self
    }
    /// Add an input event to the asciicast.
    #[allow(dead_code, reason = "Reserved for future use")]
    pub(crate) fn input(&mut self, time: u64, data: String) -> &mut Self {
        self.events.push(Event::input(time, data));
        self
    }
    /// Add a marker event to the asciicast.
    pub(crate) fn marker(&mut self, time: u64, name: String) -> &mut Self {
        self.events.push(Event::marker(time, name));
        self
    }
    /// Add a resize event to the asciicast.
    #[allow(dead_code, reason = "Reserved for future use")]
    pub(crate) fn resize(&mut self, time: u64, columns: u16, rows: u16) -> &mut Self {
        self.events.push(Event::resize(time, columns, rows));
        self
    }

    // Output
    /// Write the asciicast to a writer.
    pub fn write(&self, writer: &mut impl std::io::Write) -> serde_json::Result<()> {
        use serde_json::ser::to_writer;
        to_writer(&mut *writer, &self.header)?;
        for event in &self.events {
            writeln!(&mut *writer).map_err(serde_json::Error::io)?;
            to_writer(&mut *writer, event)?;
        }
        Ok(())
    }
}
