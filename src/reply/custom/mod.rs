//! The answers of sc-app3's pump, which scsynth never sends, one module per
//! family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod clock;
mod groups;
mod streams;

pub use clock::{Click, ClockPong, Mark, Tick};
pub use groups::{Plugin, PluginFailure, Root, RootFailure};
pub use streams::{StreamFailure, StreamOpened};
