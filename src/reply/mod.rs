//! The messages that scsynth sends to clients: the replies to commands, sent
//! to the sender of the command only, and the notifications, sent to every
//! client registered with `/notify`. scsynth never wraps them in bundles.

pub mod buffer;
pub mod command_replies;
pub mod control_bus;
pub mod custom;
pub mod group;
pub mod node_notification;
pub mod synth;
pub mod top_level;
pub mod trigger;

use crate::codec::messages;
use crate::command::top_level::SyncId;

messages! {
    /// A message from scsynth: one variant per message of the Server Command
    /// Reference that scsynth sends, except those under
    /// [Left out](crate#left-out). The answers of
    /// [sc-app3's pump](crate#the-protocol-of-sc-app3s-pump) come last.
    #[non_exhaustive]
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
        /// `/n_go`: a node started.
        "/n_go" => NGo(node_notification::NodeState),
        /// `/n_end`: a node ended, and is freed.
        "/n_end" => NEnd(node_notification::NodeState),
        /// `/n_off`: a node turned off.
        "/n_off" => NOff(node_notification::NodeState),
        /// `/n_on`: a node turned on.
        "/n_on" => NOn(node_notification::NodeState),
        /// `/n_move`: a node moved.
        "/n_move" => NMove(node_notification::NodeState),
        /// `/n_info`: the answer to `/n_query`.
        "/n_info" => NInfo(node_notification::NodeState),
        /// `/n_set`: the answer to `/s_get`.
        "/n_set" => NSet(synth::NSet),
        /// `/n_setn`: the answer to `/s_getn`.
        "/n_setn" => NSetn(synth::NSetn),
        /// `/g_queryTree.reply`: the answer to `/g_queryTree`, one per
        /// group.
        "/g_queryTree.reply" => GQueryTreeReply(group::GQueryTreeReply),
        /// `/b_info`: the answer to `/b_query`.
        "/b_info" => BInfo(buffer::BInfo),
        /// `/b_set`: the answer to `/b_get`.
        "/b_set" => BSet(buffer::BSet),
        /// `/b_setn`: the answer to `/b_getn`.
        "/b_setn" => BSetn(buffer::BSetn),
        /// `/c_set`: the answer to `/c_get`.
        "/c_set" => CSet(control_bus::CSet),
        /// `/c_setn`: the answer to `/c_getn`.
        "/c_setn" => CSetn(control_bus::CSetn),
        /// `/tr`: a `SendTrig` unit generator triggered.
        "/tr" => Tr(trigger::Tr),
        /// `/loadRoot.success`: the root group is ready.
        "/loadRoot.success" => LoadRootSuccess(custom::Root),
        /// `/loadRoot.failure`: the pump refused the load, or it failed.
        "/loadRoot.failure" => LoadRootFailure(custom::RootFailure),
        /// `/unloadRoot.success`: the root group is free.
        "/unloadRoot.success" => UnloadRootSuccess(custom::Root),
        /// `/unloadRoot.failure`: the pump refused the unload, or it failed.
        "/unloadRoot.failure" => UnloadRootFailure(custom::RootFailure),
        /// `/loadPlugin.success`: the group of the plugin is ready.
        "/loadPlugin.success" => LoadPluginSuccess(custom::Plugin),
        /// `/loadPlugin.failure`: the pump refused the load, or it failed.
        "/loadPlugin.failure" => LoadPluginFailure(custom::PluginFailure),
        /// `/unloadPlugin.success`: the group of the plugin is free.
        "/unloadPlugin.success" => UnloadPluginSuccess(custom::Plugin),
        /// `/unloadPlugin.failure`: the pump refused the unload, or it failed.
        "/unloadPlugin.failure" => UnloadPluginFailure(custom::PluginFailure),
        /// `/clock.pong`: the answer of the pump to `/clock.ping`, with the
        /// clock of the session.
        "/clock.pong" => ClockPong(custom::ClockPong),
        /// `/stream.open.success`: the stream records its bus, and the page
        /// gets its frames.
        "/stream.open.success" => StreamOpenSuccess(custom::StreamOpened),
        /// `/stream.open.failure`: the pump refused the open, or it failed.
        "/stream.open.failure" => StreamOpenFailure(custom::StreamFailure),
        /// `/stream.frames`: frames of a stream, for each of its holders.
        "/stream.frames" => StreamFrames(custom::StreamFrames),
        /// `/tick`: the clock synth of a session ticked.
        "/tick" => Tick(custom::Tick),
    }
}
