#![doc = include_str!("../README.md")]

mod argument;
mod codec;
pub mod command;
mod error;
mod packet;
mod raw;
pub mod reply;
#[cfg(test)]
mod testing;
mod types;

pub use argument::Argument;
pub use codec::Message;
pub use command::top_level::SyncId;
pub use command::{Command, Completion};
pub use error::{DecodeError, DecodeErrorKind};
pub use packet::{Bundle, Packet, Timetag};
pub use raw::{Color, MidiMessage, RawArg, RawMessage};
pub use reply::Reply;
pub use types::{
    AddAction, ArrayElement, BufferId, BusMapping, Control, ControlSetting, ControlValue,
    IndexFill, IndexRange, IndexValue, IndexValues, NewNodeId, NodeId, NotABusMapping, Number,
    OrderAction, OutOfRange,
};
