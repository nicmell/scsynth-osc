//! The messages that scsynth sends to clients: the replies to commands, sent
//! to the sender of the command only, and the notifications, sent to every
//! client registered with `/notify`. scsynth never wraps them in bundles.

pub mod buffer;
pub mod command_replies;
pub mod control_bus;
pub mod group;
pub mod node_notification;
pub mod synth;
pub mod top_level;
pub mod trigger;

use crate::command::top_level::SyncId;

crate::messages! {
    @cfg [feature = "serde"]
    /// A message from scsynth: one variant per message of the Server Command
    /// Reference that scsynth sends, except those under
    /// [Left out](crate#left-out).
    #[non_exhaustive]
    pub enum Reply {
        /// `/done`: an asynchronous command completed.
        "/done" as DONE => Done(command_replies::Done),
        /// `/fail`: a command failed.
        "/fail" as FAIL => Fail(command_replies::Fail),
        /// `/status.reply`: the answer to `/status`.
        "/status.reply" as STATUS_REPLY => StatusReply(top_level::StatusReply),
        /// `/version.reply`: the answer to `/version`.
        "/version.reply" as VERSION_REPLY => VersionReply(top_level::VersionReply),
        /// `/rtMemoryStatus.reply`: the answer to `/rtMemoryStatus`.
        "/rtMemoryStatus.reply" as RT_MEMORY_STATUS_REPLY =>
            RtMemoryStatusReply(top_level::RtMemoryStatusReply),
        /// `/synced`: the answer to `/sync`, once every asynchronous command
        /// received before it completed.
        "/synced" as SYNCED => Synced(SyncId),
        /// `/n_go`: a node started.
        "/n_go" as N_GO => NGo(node_notification::NodeState),
        /// `/n_end`: a node ended, and is freed.
        "/n_end" as N_END => NEnd(node_notification::NodeState),
        /// `/n_off`: a node turned off.
        "/n_off" as N_OFF => NOff(node_notification::NodeState),
        /// `/n_on`: a node turned on.
        "/n_on" as N_ON => NOn(node_notification::NodeState),
        /// `/n_move`: a node moved.
        "/n_move" as N_MOVE => NMove(node_notification::NodeState),
        /// `/n_info`: the answer to `/n_query`.
        "/n_info" as N_INFO => NInfo(node_notification::NodeState),
        /// `/n_set`: the answer to `/s_get`.
        "/n_set" as N_SET => NSet(synth::NSet),
        /// `/n_setn`: the answer to `/s_getn`.
        "/n_setn" as N_SETN => NSetn(synth::NSetn),
        /// `/g_queryTree.reply`: the answer to `/g_queryTree`, one per
        /// group.
        "/g_queryTree.reply" as G_QUERY_TREE_REPLY => GQueryTreeReply(group::GQueryTreeReply),
        /// `/b_info`: the answer to `/b_query`.
        "/b_info" as B_INFO => BInfo(buffer::BInfo),
        /// `/b_set`: the answer to `/b_get`.
        "/b_set" as B_SET => BSet(buffer::BSet),
        /// `/b_setn`: the answer to `/b_getn`.
        "/b_setn" as B_SETN => BSetn(buffer::BSetn),
        /// `/c_set`: the answer to `/c_get`.
        "/c_set" as C_SET => CSet(control_bus::CSet),
        /// `/c_setn`: the answer to `/c_getn`.
        "/c_setn" as C_SETN => CSetn(control_bus::CSetn),
        /// `/tr`: a `SendTrig` unit generator triggered.
        "/tr" as TR => Tr(trigger::Tr),
    }
}
