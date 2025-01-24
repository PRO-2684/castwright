//! Module for serializing an [asciicast v2 event](https://docs.asciinema.org/manual/asciicast/v2/#event-stream).

use serde::Serialize;

/// The event of an asciicast v2 file.
// From https://github.com/asciinema/asciinema/blob/f0f908872ca0364128b546bcc8af918d2fc47566/src/asciicast/v2.rs#L38-L45
#[derive(Debug)]
pub struct Event {
    /// Indicates when the event happened, represented as the number of milliseconds since the beginning of the recording session.
    time: u64,
    /// Type of the event.
    code: EventCode,
    /// Event specific data, described separately for each event code.
    data: String,
}

impl Serialize for Event {
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

impl Event {
    /// Create a new output event.
    pub fn output(time: u64, data: String) -> Self {
        Self {
            time,
            code: EventCode::Output,
            data,
        }
    }
    /// Create a new input event.
    pub fn input(time: u64, data: String) -> Self {
        Self {
            time,
            code: EventCode::Input,
            data,
        }
    }
    /// Create a new marker event.
    pub fn marker(time: u64, name: String) -> Self {
        Self {
            time,
            code: EventCode::Marker,
            data: name,
        }
    }
    /// Create a new resize event.
    pub fn resize(time: u64, columns: u16, rows: u16) -> Self {
        Self {
            time,
            code: EventCode::Resize,
            data: format!("{}x{}", columns, rows),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialize() -> serde_json::Result<()> {
        let event = Event {
            time: 1_002_345,
            code: EventCode::Output,
            data: "Hello, world!".to_string(),
        };
        let expected = r#"[1.002345,"o","Hello, world!"]"#;
        let serialized = serde_json::to_string(&event)?;
        assert_eq!(serialized, expected);
        Ok(())
    }
}
