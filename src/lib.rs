//! The OSC protocol of scsynth, the synthesis server of SuperCollider: the
//! commands and the replies of the Server Command Reference, as Rust types
//! that encode to the bytes that scsynth reads, and decode from the bytes
//! that it writes.
//!
//! ```
//! use scsynth_osc::{Command, Reply};
//! use scsynth_osc::command::top_level::SyncId;
//!
//! // A client asks scsynth to answer once its pending work is done…
//! let bytes = Command::Sync(SyncId(7)).encode();
//! assert_eq!(Command::decode(&bytes)?, Command::Sync(SyncId(7)));
//!
//! // …and scsynth answers /synced 7.
//! let answer = Reply::Synced(SyncId(7)).encode();
//! assert_eq!(Reply::decode(&answer)?, Reply::Synced(SyncId(7)));
//! # Ok::<(), scsynth_osc::DecodeError>(())
//! ```
//!
//! [`Command`] has one variant per command, and [`Reply`] one per message
//! that scsynth sends. Both encode to one OSC message, and decode from one.
//! A [`Packet`] adds bundles. The modules [`command`] and [`reply`] follow
//! the sections of the reference.
//!
//! # Strictness
//!
//! The crate follows what scsynth 3.14.1 does, where its code and the
//! reference differ. The README lists each difference.
//!
//! - Encoding writes each argument in the one type that the reference gives
//!   it. An optional argument is left out only at the end of a message.
//! - Decoding takes each argument only in that type, and refuses a message
//!   with an argument too many or too few, or with a value that the
//!   protocol does not know. scsynth itself converts many types, so it
//!   reads more than this crate decodes.
//! - No value of these types encodes to bytes that make scsynth misread a
//!   message or hang.
//!
//! # Features
//!
//! - `serde`: `Serialize` and `Deserialize` for every type. A command or a
//!   reply is then an object with its `address` and its `args`.

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

pub use argument::Argument;
pub use command::{Command, Completion};
pub use error::{DecodeError, DecodeErrorKind};
pub use packet::{Bundle, Packet, Timetag};
pub use reply::Reply;
