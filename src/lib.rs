#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod argument;
mod codec;
pub mod command;
mod error;
mod packet;
pub mod reply;
#[cfg(test)]
mod testing;
mod types;

pub use argument::Argument;
pub use command::{Command, Completion};
pub use error::{DecodeError, DecodeErrorKind};
pub use packet::{Bundle, Packet, Timetag};
pub use reply::Reply;
pub use types::{
    AddAction, ArrayElement, BusMapping, Control, ControlSetting, ControlValue, NewNodeId, NodeId,
    NotABusMapping, Number, OrderAction,
};
