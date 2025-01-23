//! Module for serializing to [asciicast v2 format](https://docs.asciinema.org/manual/asciicast/v2/).

mod event;
mod header;
pub use event::Event;
pub use header::Header;
