//! The answers of sc-app3's pump, which scsynth never sends, one module per
//! family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod groups;

pub use groups::{Plugin, PluginFailure, Root, RootFailure};
