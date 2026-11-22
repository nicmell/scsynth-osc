//! The answers of sc-app3's pump, which scsynth never sends, one module per
//! family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod clock;
mod dirt;
mod groups;
mod nodes;
mod states;
mod streams;
mod synthdefs;
mod tempo;

pub use clock::{ClockPong, Tick};
pub use dirt::{Sample, SampleFailure};
pub use groups::{Plugin, PluginFailure, RootFailure};
pub use nodes::{Node, NodeFailure, NodeLoaded};
pub use states::States;
pub use streams::{StreamFailure, StreamFrames, StreamOpened};
pub use synthdefs::{Synthdef, SynthdefFailure};
pub use tempo::Tempo;
