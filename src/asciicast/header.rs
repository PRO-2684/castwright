//! Module for modeling and serializing an [asciicast v2 header](https://docs.asciinema.org/manual/asciicast/v2/#header).

use crate::util::{capture_env_vars, get_terminal_size};
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::collections::HashMap;

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
    /// Unix timestamp of the beginning of the recording session in seconds.
    pub timestamp: Option<u64>,
    /// Idle time limit.
    pub idle_time_limit: Option<f64>,
    /// Title of the asciicast.
    pub title: Option<String>,
    /// Map of captured environment variables.
    pub env: Option<HashMap<String, String>>,
    // Not implemented fields:
    // Duration of the whole recording in seconds (when it's known upfront).
    // duration: Option<u64>,
    // Command that was recorded.
    // command: Option<String>,
    // Color theme of the recorded terminal.
    // theme: Option<V2Theme>,
}

fn serialize_or_skip<S, T>(
    state: &mut S,
    key: &'static str,
    value: &Option<T>,
) -> Result<(), S::Error>
where
    S: SerializeStruct,
    T: Serialize,
{
    if let Some(value) = value {
        state.serialize_field(key, value)?;
    } else {
        state.skip_field(key)?;
    }
    Ok(())
}

impl Serialize for Header {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
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
        if self.env.is_some() {
            len += 1;
        }

        let mut state = serializer.serialize_struct("Header", len)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;

        // Skip `None` fields
        serialize_or_skip(&mut state, "timestamp", &self.timestamp)?;
        serialize_or_skip(&mut state, "idle_time_limit", &self.idle_time_limit)?;
        serialize_or_skip(&mut state, "title", &self.title)?;
        serialize_or_skip(&mut state, "env", &self.env)?;

        state.end()
    }
}

impl Header {
    /// Create a new header with default width and height.
    pub fn new() -> Self {
        let (width, height) = get_terminal_size();
        Header {
            version: 2,
            width,
            height,
            timestamp: None,
            idle_time_limit: None,
            title: None,
            env: Some(capture_env_vars(vec![
                "SHELL".to_string(),
                "TERM".to_string(),
            ])),
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
            env: None,
        };
        let expected =
            r#"{"version":2,"width":80,"height":24,"idle_time_limit":2.0,"title":"My asciicast"}"#;
        assert_eq!(serde_json::to_string(&header)?, expected);
        Ok(())
    }
}
