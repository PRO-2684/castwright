//! Module for serializing an [asciicast v2 event](https://docs.asciinema.org/manual/asciicast/v2/#event-stream).

use serde::Serialize;

/// The event of an asciicast v2 file.
// From https://github.com/asciinema/asciinema/blob/f0f908872ca0364128b546bcc8af918d2fc47566/src/asciicast/v2.rs#L38-L45
#[derive(Debug)]
pub(super) struct Event<'a> {
    /// Indicates when the event happened, represented as the number of milliseconds since the beginning of the recording session.
    time: u64,
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
    pub fn output(time: u64, data: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Output,
            data,
        }
    }
    /// Create a new input event.
    pub fn input(time: u64, data: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Input,
            data,
        }
    }
    /// Create a new marker event.
    pub fn marker(time: u64, name: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Marker,
            data: name,
        }
    }
    /// Create a new resize event.
    pub fn resize(time: u64, dim: &'a str) -> Self {
        Self {
            time,
            code: EventCode::Resize,
            data: dim,
        }
    }
    /// Write the event to the writer.
    pub fn write(&self, writer: impl std::io::Write) -> Result<(), serde_json::Error> {
        use serde::Serialize;
        let mut serializer = serde_json::Serializer::with_formatter(writer, Formatter);
        self.serialize(&mut serializer)?;
        Ok(())
    }
}

/// Type of an event.
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
        // Write the value with 6 decimal places.
        write!(writer, "{value:.6}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialize() -> serde_json::Result<()> {
        let event = Event {
            time: 1_002_000,
            code: EventCode::Output,
            data: "Hello, world!",
        };
        let expected = r#"[1.002000,"o","Hello, world!"]"#;
        let mut serialized = Vec::new();
        event.write(&mut serialized)?;
        let serialized = String::from_utf8(serialized).unwrap();
        assert_eq!(serialized, expected);
        Ok(())
    }
}
