//! The commands that a client sends to scsynth, in the sections of the
//! Server Command Reference.

pub mod buffer;
pub mod buffer_fill;
pub mod control_bus;
pub mod group;
pub mod node;
pub mod synth;
pub mod synth_def;
pub mod top_level;
pub mod unit_generator;

use rosc::{OscBundle, OscPacket, OscType};

use crate::codec::sealed::Codec;
use crate::codec::{Reader, Writer, messages};
use crate::error::{DecodeError, DecodeErrorKind};
use crate::packet::{Timetag, decode_osc, encode_osc};

messages! {
    /// A command for scsynth: one variant per command of the Server Command
    /// Reference, except those under [Left out](crate#left-out).
    #[non_exhaustive]
    Command {
        /// `/quit`: the server quits. It replies `/done /quit`.
        "/quit" => Quit,
        /// `/notify`: registers the sender for notifications, or ends its
        /// registration.
        "/notify" => Notify(top_level::Notify),
        /// `/status`: the server replies `/status.reply`.
        "/status" => Status,
        /// `/cmd`: a command that a plugin defines.
        "/cmd" => Cmd(top_level::Cmd),
        /// `/dumpOSC`: the server prints the messages it receives, or stops.
        "/dumpOSC" => DumpOsc(top_level::DumpMode),
        /// `/sync`: the server replies `/synced` once every asynchronous
        /// command received before it completed.
        "/sync" => Sync(top_level::SyncId),
        /// `/clearSched`: the server drops every scheduled bundle.
        "/clearSched" => ClearSched,
        /// `/error`: turns the posting of errors on or off.
        "/error" => Error(top_level::ErrorMode),
        /// `/version`: the server replies `/version.reply`.
        "/version" => Version,
        /// `/rtMemoryStatus`: the server replies `/rtMemoryStatus.reply`.
        "/rtMemoryStatus" => RtMemoryStatus,
        /// `/d_recv`: defines synths from the bytes of a synth definition
        /// file.
        "/d_recv" => DRecv(synth_def::DRecv),
        /// `/d_load`: loads synth definitions from files.
        "/d_load" => DLoad(synth_def::DLoad),
        /// `/d_loadDir`: loads the synth definitions of a directory.
        "/d_loadDir" => DLoadDir(synth_def::DLoadDir),
        /// `/d_free`: removes synth definitions.
        "/d_free" => DFree(synth_def::DFree),
        /// `/n_free`: frees nodes.
        "/n_free" => NFree(node::NFree),
        /// `/n_run`: turns nodes on or off.
        "/n_run" => NRun(node::NRun),
        /// `/n_set`: sets controls of a node.
        "/n_set" => NSet(node::NSet),
        /// `/n_setn`: sets ranges of controls of a node.
        "/n_setn" => NSetn(node::NSetn),
        /// `/n_fill`: sets ranges of controls of a node to one value each.
        "/n_fill" => NFill(node::NFill),
        /// `/n_map`: maps controls of a node to control buses.
        "/n_map" => NMap(node::NMap),
        /// `/n_mapn`: maps ranges of controls of a node to control buses.
        "/n_mapn" => NMapn(node::NMapn),
        /// `/n_mapa`: maps controls of a node to audio buses.
        "/n_mapa" => NMapa(node::NMapa),
        /// `/n_mapan`: maps ranges of controls of a node to audio buses.
        "/n_mapan" => NMapan(node::NMapan),
        /// `/n_before`: moves nodes before others.
        "/n_before" => NBefore(node::NBefore),
        /// `/n_after`: moves nodes after others.
        "/n_after" => NAfter(node::NAfter),
        /// `/n_query`: the server sends `/n_info` for nodes.
        "/n_query" => NQuery(node::NQuery),
        /// `/n_trace`: the server prints what nodes do for one control
        /// period.
        "/n_trace" => NTrace(node::NTrace),
        /// `/n_order`: moves nodes next to a target, in order.
        "/n_order" => NOrder(node::NOrder),
        /// `/s_new`: creates a synth.
        "/s_new" => SNew(synth::SNew),
        /// `/s_get`: the server replies `/n_set` with values of controls.
        "/s_get" => SGet(synth::SGet),
        /// `/s_getn`: the server replies `/n_setn` with values of ranges of
        /// controls.
        "/s_getn" => SGetn(synth::SGetn),
        /// `/s_noid`: synths give up their IDs.
        "/s_noid" => SNoid(synth::SNoid),
        /// `/g_new`: creates groups.
        "/g_new" => GNew(group::GNew),
        /// `/p_new`: creates parallel groups.
        "/p_new" => PNew(group::PNew),
        /// `/g_head`: moves nodes to the head of groups.
        "/g_head" => GHead(group::GHead),
        /// `/g_tail`: moves nodes to the tail of groups.
        "/g_tail" => GTail(group::GTail),
        /// `/g_freeAll`: frees every node in groups.
        "/g_freeAll" => GFreeAll(group::GFreeAll),
        /// `/g_deepFree`: frees every synth in groups, and in the groups
        /// inside them.
        "/g_deepFree" => GDeepFree(group::GDeepFree),
        /// `/g_dumpTree`: the server prints the trees of groups.
        "/g_dumpTree" => GDumpTree(group::GDumpTree),
        /// `/g_queryTree`: the server replies `/g_queryTree.reply` for
        /// groups.
        "/g_queryTree" => GQueryTree(group::GQueryTree),
        /// `/u_cmd`: a command that a plugin defines for a unit generator.
        "/u_cmd" => UCmd(unit_generator::UCmd),
        /// `/b_alloc`: allocates a buffer.
        "/b_alloc" => BAlloc(buffer::BAlloc),
        /// `/b_allocRead`: allocates a buffer, and reads a sound file into
        /// it.
        "/b_allocRead" => BAllocRead(buffer::BAllocRead),
        /// `/b_allocReadChannel`: allocates a buffer, and reads channels of
        /// a sound file into it.
        "/b_allocReadChannel" => BAllocReadChannel(buffer::BAllocReadChannel),
        /// `/b_read`: reads a sound file into a buffer.
        "/b_read" => BRead(buffer::BRead),
        /// `/b_readChannel`: reads channels of a sound file into a buffer.
        "/b_readChannel" => BReadChannel(buffer::BReadChannel),
        /// `/b_write`: writes a buffer into a sound file.
        "/b_write" => BWrite(buffer::BWrite),
        /// `/b_free`: frees a buffer.
        "/b_free" => BFree(buffer::BFree),
        /// `/b_zero`: sets every sample of a buffer to 0.
        "/b_zero" => BZero(buffer::BZero),
        /// `/b_set`: sets samples of a buffer.
        "/b_set" => BSet(buffer::BSet),
        /// `/b_setn`: sets ranges of samples of a buffer.
        "/b_setn" => BSetn(buffer::BSetn),
        /// `/b_setSampleRate`: sets the sample rate of a buffer.
        "/b_setSampleRate" => BSetSampleRate(buffer::BSetSampleRate),
        /// `/b_fill`: sets ranges of samples of a buffer to one value each.
        "/b_fill" => BFill(buffer::BFill),
        /// `/b_gen`: a fill command writes a buffer.
        "/b_gen" => BGen(buffer::BGen),
        /// `/b_close`: closes the sound file of a buffer.
        "/b_close" => BClose(buffer::BClose),
        /// `/b_query`: the server replies `/b_info` for buffers.
        "/b_query" => BQuery(buffer::BQuery),
        /// `/b_get`: the server replies `/b_set` with values of samples.
        "/b_get" => BGet(buffer::BGet),
        /// `/b_getn`: the server replies `/b_setn` with values of ranges of
        /// samples.
        "/b_getn" => BGetn(buffer::BGetn),
        /// `/c_set`: sets control buses.
        "/c_set" => CSet(control_bus::CSet),
        /// `/c_setn`: sets ranges of control buses.
        "/c_setn" => CSetn(control_bus::CSetn),
        /// `/c_fill`: sets ranges of control buses to one value each.
        "/c_fill" => CFill(control_bus::CFill),
        /// `/c_get`: the server replies `/c_set` with values of buses.
        "/c_get" => CGet(control_bus::CGet),
        /// `/c_getn`: the server replies `/c_setn` with values of ranges of
        /// buses.
        "/c_getn" => CGetn(control_bus::CGetn),
    }
}

/// What an asynchronous command runs once its work is done. It travels as
/// a blob in the command, and its replies go to the sender of the command.
/// See [Completion messages](crate#completion-messages) for when it runs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Completion {
    /// One command.
    Message(Box<Command>),
    /// Commands that run together. scsynth runs them at once when the time
    /// tag is immediate, and schedules them otherwise. A bundle cannot hold
    /// a bundle here: scsynth fails on one.
    Bundle {
        /// When the commands run.
        time: Timetag,
        /// The commands, in the order they run.
        commands: Vec<Command>,
    },
}

impl Completion {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Message(command) => encode_osc(&OscPacket::Message(command.to_osc())),
            Self::Bundle { time, commands } => encode_osc(&OscPacket::Bundle(OscBundle {
                timetag: time.to_osc(),
                content: commands
                    .iter()
                    .map(|command| OscPacket::Message(command.to_osc()))
                    .collect(),
            })),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        match decode_osc(bytes)? {
            OscPacket::Message(message) => {
                Command::from_osc(&message).map(|command| Self::Message(Box::new(command)))
            }
            OscPacket::Bundle(bundle) => Ok(Self::Bundle {
                time: Timetag::from_osc(bundle.timetag),
                commands: bundle
                    .content
                    .iter()
                    .map(|packet| match packet {
                        OscPacket::Message(message) => Command::from_osc(message),
                        OscPacket::Bundle(_) => Err(DecodeError::new("", DecodeErrorKind::Bundle)),
                    })
                    .collect::<Result<_, _>>()?,
            }),
        }
    }
}

/// Writes the completion message, when there is one.
pub(crate) fn write_completion(writer: &mut Writer, completion: Option<&Completion>) {
    if let Some(completion) = completion {
        writer.blob(completion.to_bytes());
    }
}

/// Reads the completion message at the reader's position, when there is
/// one. With `placeholder`, an int 0 there stands for none: scsynth skips
/// one int in this place, and sclang sends 0 for no completion message.
pub(crate) fn read_completion(
    reader: &mut Reader<'_>,
    placeholder: bool,
) -> Result<Option<Completion>, DecodeError> {
    let index = reader.index();
    match reader.peek() {
        Some(OscType::Blob(_)) => {
            let bytes = reader.blob()?;
            if bytes.is_empty() {
                return Err(reader.invalid(
                    index,
                    "an empty completion message, which scsynth leaves unread",
                ));
            }
            Completion::from_bytes(&bytes).map(Some).map_err(|error| {
                reader.error(DecodeErrorKind::Completion {
                    index,
                    error: Box::new(error),
                })
            })
        }
        Some(OscType::Int(_)) if placeholder => match reader.int()? {
            0 => Ok(None),
            other => Err(reader.invalid(
                index,
                format!("{other} in place of a completion message, where only 0 stands for none"),
            )),
        },
        _ => Ok(None),
    }
}
