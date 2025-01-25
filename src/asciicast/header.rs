//! Module for serializing an [asciicast v2 header](https://docs.asciinema.org/manual/asciicast/v2/#header).

use crate::util::get_terminal_size;
use serde::Serialize;

/// The header of an asciicast v2 file.
// From: https://github.com/asciinema/asciinema/blob/f0f908872ca0364128b546bcc8af918d2fc47566/src/asciicast/v2.rs##L9-L20))
#[derive(Debug)]
pub(super) struct Header {
    /// The version of the asciicast format. Must be set to 2.
    version: u8,
    /// Initial terminal width, i.e number of columns.
    pub width: u16,
    /// Initial terminal height, i.e number of rows.
    pub height: u16,
    /// Unix timestamp of the beginning of the recording session.
    pub timestamp: Option<u64>,
    /// Idle time limit.
    pub idle_time_limit: Option<f64>,
    /// Title of the asciicast.
    pub title: Option<String>,
    // Not implemented fields:
    // Duration of the whole recording in seconds (when it's known upfront).
    // duration: Option<u64>,
    // Command that was recorded.
    // command: Option<String>,
    // Map of captured environment variables.
    // env: Option<HashMap<String, String>>,
    // Color theme of the recorded terminal.
    // theme: Option<V2Theme>,
}

impl Serialize for Header {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        // Count length of fields
        let mut len = 3;
        if self.timestamp.is_some() {
            len += 1;
        }
        if self.idle_time_limit.is_some() {
            len += 1;
        }
        if self.title.is_some() {
            len += 1;
        }

        // Skip `None` fields
        let mut state = serializer.serialize_struct("Header", len)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        if let Some(timestamp) = self.timestamp {
            state.serialize_field("timestamp", &timestamp)?;
        } else {
            state.skip_field("timestamp")?;
        };
        if let Some(idle_time_limit) = self.idle_time_limit {
            state.serialize_field("idle_time_limit", &idle_time_limit)?;
        } else {
            state.skip_field("idle_time_limit")?;
        };
        if let Some(title) = &self.title {
            state.serialize_field("title", title)?;
        } else {
            state.skip_field("title")?;
        };
        state.end()
    }
}

impl Header {
    /// Create a new header with the given width and height.
    pub fn new() -> Self {
        let (width, height) = get_terminal_size();
        Header {
            version: 2,
            width,
            height,
            timestamp: None,
            idle_time_limit: None,
            title: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_serialize() -> serde_json::Result<()> {
        let header = Header {
            version: 2,
            width: 80,
            height: 24,
            timestamp: None,
            idle_time_limit: Some(2.0),
            title: Some("My asciicast".to_string()),
        };
        let expected =
            r#"{"version":2,"width":80,"height":24,"idle_time_limit":2.0,"title":"My asciicast"}"#;
        assert_eq!(serde_json::to_string(&header)?, expected);
        Ok(())
    }
}
