//! Module for modeling and serializing an [asciicast v2 event](https://docs.asciinema.org/manual/asciicast/v2/#event-stream).

use serde::Serialize;
use std::io::Write;

/// An event of an asciicast v2 file.
// Adapted from https://github.com/asciinema/asciinema/blob/f0f908872ca0364128b546bcc8af918d2fc47566/src/asciicast/v2.rs#L38-L45
#[derive(Debug)]
pub(super) struct Event<'a> {
    /// Indicates when the event happened, represented as the number of milliseconds since the beginning of the recording session.
    time: u128,
    /// Type of the event.
    code: EventCode,
    /// Event specific data, described separately for each event code.
    data: &'a str,
}

impl Serialize for Event<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        // Note that an `Event` is serialized as an array, instead of an object.
        let mut state = serializer.serialize_seq(Some(3))?;
        let time = self.time as f64 / 1_000_000.0;
        state.serialize_element(&time)?;
        state.serialize_element(&self.code)?;
        state.serialize_element(&self.data)?;
        state.end()
    }
}

impl<'a> Event<'a> {
    /// Create a new output event.
    pub fn output(time: u128, data: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Output,
            data,
        }
    }
    /// Create a new input event.
    pub fn input(time: u128, data: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Input,
            data,
        }
    }
    /// Create a new marker event.
    pub fn marker(time: u128, name: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Marker,
            data: name,
        }
    }
    /// Create a new resize event.
    pub fn resize(time: u128, dim: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Resize,
            data: dim,
        }
    }
    /// Write the event to the writer.
    pub fn write(&self, writer: impl Write) -> Result<(), serde_json::Error> {
        let mut serializer = serde_json::Serializer::with_formatter(writer, Formatter);
        self.serialize(&mut serializer)?;
        Ok(())
    }
}

/// Possible types of an event.
// From: https://github.com/asciinema/asciinema/blob/f0f908872ca0364128b546bcc8af918d2fc47566/src/asciicast/v2.rs#L47-L54))
#[derive(Debug)]
enum EventCode {
    Output,
    Input,
    Marker,
    Resize,
}

impl Serialize for EventCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use EventCode::*;

        match self {
            Output => serializer.serialize_str("o"),
            Input => serializer.serialize_str("i"),
            Marker => serializer.serialize_str("m"),
            Resize => serializer.serialize_str("r"),
        }
    }
}

/// Formatter that writes floating point numbers with 6 decimal places.
struct Formatter;

impl serde_json::ser::Formatter for Formatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        write!(writer, "{value:.6}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialize() -> serde_json::Result<()> {
        let pairs = [
            (
                Event::output(0, "Output event"),
                r#"[0.000000,"o","Output event"]"#,
            ),
            (
                Event::input(1_000_000, "Input event"),
                r#"[1.000000,"i","Input event"]"#,
            ),
            (
                Event::marker(1_000_001, "Marker event"),
                r#"[1.000001,"m","Marker event"]"#,
            ),
            (
                Event::resize(1_002_000, "80x24"),
                r#"[1.002000,"r","80x24"]"#,
            ),
        ];

        for (event, expected) in pairs.iter() {
            let mut serialized = Vec::new();
            event.write(&mut serialized)?;
            let serialized = String::from_utf8(serialized).unwrap();
            assert_eq!(serialized, *expected);
        }

        Ok(())
    }
}
