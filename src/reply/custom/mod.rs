//! The answers of sc-app3's pump, which scsynth never sends, one module per
//! family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod clock;
mod groups;
mod nodes;
mod streams;
mod synthdefs;

pub use clock::{ClockPong, Tick};
pub use groups::{Plugin, PluginFailure, Root, RootFailure};
pub use nodes::{Node, NodeFailure, NodeLoaded};
pub use streams::{StreamFailure, StreamFrames, StreamOpened};
pub use synthdefs::{Synthdef, SynthdefFailure};
