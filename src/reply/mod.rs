//! The messages that scsynth sends to clients: the replies to commands, sent
//! to the sender of the command only, and the notifications, sent to every
//! client registered with `/notify`. scsynth never wraps them in bundles.

pub mod command_replies;
pub mod top_level;

use crate::codec::messages;
use crate::command::top_level::SyncId;

messages! {
    /// A message from scsynth: one variant per address.
    Reply {
        /// `/done`: an asynchronous command completed.
        "/done" => Done(command_replies::Done),
        /// `/fail`: a command failed.
        "/fail" => Fail(command_replies::Fail),
        /// `/status.reply`: the answer to `/status`.
        "/status.reply" => StatusReply(top_level::StatusReply),
        /// `/version.reply`: the answer to `/version`.
        "/version.reply" => VersionReply(top_level::VersionReply),
        /// `/rtMemoryStatus.reply`: the answer to `/rtMemoryStatus`.
        "/rtMemoryStatus.reply" => RtMemoryStatusReply(top_level::RtMemoryStatusReply),
        /// `/synced`: the answer to `/sync`, once every asynchronous command
        /// received before it completed.
        "/synced" => Synced(SyncId),
    }
}
