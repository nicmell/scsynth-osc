//! The answers of sc-app3's pump, which scsynth never sends, one module per
//! family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod clock;
mod dirt;
mod groups;
mod streams;
mod tempo;

pub use clock::{ClockPong, Tick};
pub use dirt::{Dirt, DirtFailure};
pub use groups::{Plugin, PluginFailure};
pub use streams::{StreamFailure, StreamFrames, StreamOpened};
pub use tempo::Tempo;
