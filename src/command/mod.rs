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
    pub enum Command {
        /// `/quit`: the server quits. It replies `/done /quit`.
        "/quit" as QUIT => Quit,
        /// `/notify`: registers the sender for notifications, or ends its
        /// registration.
        "/notify" as NOTIFY => Notify(top_level::Notify),
        /// `/status`: the server replies `/status.reply`.
        "/status" as STATUS => Status,
        /// `/cmd`: a command that a plugin defines.
        "/cmd" as CMD => Cmd(top_level::Cmd),
        /// `/dumpOSC`: the server prints the messages it receives, or stops.
        "/dumpOSC" as DUMP_OSC => DumpOsc(top_level::DumpMode),
        /// `/sync`: the server replies `/synced` once every asynchronous
        /// command received before it completed.
        "/sync" as SYNC => Sync(top_level::SyncId),
        /// `/clearSched`: the server drops every scheduled bundle.
        "/clearSched" as CLEAR_SCHED => ClearSched,
        /// `/error`: turns the posting of errors on or off.
        "/error" as ERROR => Error(top_level::ErrorMode),
        /// `/version`: the server replies `/version.reply`.
        "/version" as VERSION => Version,
        /// `/rtMemoryStatus`: the server replies `/rtMemoryStatus.reply`.
        "/rtMemoryStatus" as RT_MEMORY_STATUS => RtMemoryStatus,
        /// `/d_recv`: defines synths from the bytes of a synth definition
        /// file.
        "/d_recv" as D_RECV => DRecv(synth_def::DRecv),
        /// `/d_load`: loads synth definitions from files.
        "/d_load" as D_LOAD => DLoad(synth_def::DLoad),
        /// `/d_loadDir`: loads the synth definitions of a directory.
        "/d_loadDir" as D_LOAD_DIR => DLoadDir(synth_def::DLoadDir),
        /// `/d_free`: removes synth definitions.
        "/d_free" as D_FREE => DFree(synth_def::DFree),
        /// `/n_free`: frees nodes.
        "/n_free" as N_FREE => NFree(node::NFree),
        /// `/n_run`: turns nodes on or off.
        "/n_run" as N_RUN => NRun(node::NRun),
        /// `/n_set`: sets controls of a node.
        "/n_set" as N_SET => NSet(node::NSet),
        /// `/n_setn`: sets ranges of controls of a node.
        "/n_setn" as N_SETN => NSetn(node::NSetn),
        /// `/n_fill`: sets ranges of controls of a node to one value each.
        "/n_fill" as N_FILL => NFill(node::NFill),
        /// `/n_map`: maps controls of a node to control buses.
        "/n_map" as N_MAP => NMap(node::NMap),
        /// `/n_mapn`: maps ranges of controls of a node to control buses.
        "/n_mapn" as N_MAPN => NMapn(node::NMapn),
        /// `/n_mapa`: maps controls of a node to audio buses.
        "/n_mapa" as N_MAPA => NMapa(node::NMapa),
        /// `/n_mapan`: maps ranges of controls of a node to audio buses.
        "/n_mapan" as N_MAPAN => NMapan(node::NMapan),
        /// `/n_before`: moves nodes before others.
        "/n_before" as N_BEFORE => NBefore(node::NBefore),
        /// `/n_after`: moves nodes after others.
        "/n_after" as N_AFTER => NAfter(node::NAfter),
        /// `/n_query`: the server sends `/n_info` for nodes.
        "/n_query" as N_QUERY => NQuery(node::NQuery),
        /// `/n_trace`: the server prints what nodes do for one control
        /// period.
        "/n_trace" as N_TRACE => NTrace(node::NTrace),
        /// `/n_order`: moves nodes next to a target, in order.
        "/n_order" as N_ORDER => NOrder(node::NOrder),
        /// `/s_new`: creates a synth.
        "/s_new" as S_NEW => SNew(synth::SNew),
        /// `/s_get`: the server replies `/n_set` with values of controls.
        "/s_get" as S_GET => SGet(synth::SGet),
        /// `/s_getn`: the server replies `/n_setn` with values of ranges of
        /// controls.
        "/s_getn" as S_GETN => SGetn(synth::SGetn),
        /// `/s_noid`: synths give up their IDs.
        "/s_noid" as S_NOID => SNoid(synth::SNoid),
        /// `/g_new`: creates groups.
        "/g_new" as G_NEW => GNew(group::GNew),
        /// `/p_new`: creates parallel groups.
        "/p_new" as P_NEW => PNew(group::PNew),
        /// `/g_head`: moves nodes to the head of groups.
        "/g_head" as G_HEAD => GHead(group::GHead),
        /// `/g_tail`: moves nodes to the tail of groups.
        "/g_tail" as G_TAIL => GTail(group::GTail),
        /// `/g_freeAll`: frees every node in groups.
        "/g_freeAll" as G_FREE_ALL => GFreeAll(group::GFreeAll),
        /// `/g_deepFree`: frees every synth in groups, and in the groups
        /// inside them.
        "/g_deepFree" as G_DEEP_FREE => GDeepFree(group::GDeepFree),
        /// `/g_dumpTree`: the server prints the trees of groups.
        "/g_dumpTree" as G_DUMP_TREE => GDumpTree(group::GDumpTree),
        /// `/g_queryTree`: the server replies `/g_queryTree.reply` for
        /// groups.
        "/g_queryTree" as G_QUERY_TREE => GQueryTree(group::GQueryTree),
        /// `/u_cmd`: a command that a plugin defines for a unit generator.
        "/u_cmd" as U_CMD => UCmd(unit_generator::UCmd),
        /// `/b_alloc`: allocates a buffer.
        "/b_alloc" as B_ALLOC => BAlloc(buffer::BAlloc),
        /// `/b_allocRead`: allocates a buffer, and reads a sound file into
        /// it.
        "/b_allocRead" as B_ALLOC_READ => BAllocRead(buffer::BAllocRead),
        /// `/b_allocReadChannel`: allocates a buffer, and reads channels of
        /// a sound file into it.
        "/b_allocReadChannel" as B_ALLOC_READ_CHANNEL =>
            BAllocReadChannel(buffer::BAllocReadChannel),
        /// `/b_read`: reads a sound file into a buffer.
        "/b_read" as B_READ => BRead(buffer::BRead),
        /// `/b_readChannel`: reads channels of a sound file into a buffer.
        "/b_readChannel" as B_READ_CHANNEL => BReadChannel(buffer::BReadChannel),
        /// `/b_write`: writes a buffer into a sound file.
        "/b_write" as B_WRITE => BWrite(buffer::BWrite),
        /// `/b_free`: frees a buffer.
        "/b_free" as B_FREE => BFree(buffer::BFree),
        /// `/b_zero`: sets every sample of a buffer to 0.
        "/b_zero" as B_ZERO => BZero(buffer::BZero),
        /// `/b_set`: sets samples of a buffer.
        "/b_set" as B_SET => BSet(buffer::BSet),
        /// `/b_setn`: sets ranges of samples of a buffer.
        "/b_setn" as B_SETN => BSetn(buffer::BSetn),
        /// `/b_setSampleRate`: sets the sample rate of a buffer.
        "/b_setSampleRate" as B_SET_SAMPLE_RATE => BSetSampleRate(buffer::BSetSampleRate),
        /// `/b_fill`: sets ranges of samples of a buffer to one value each.
        "/b_fill" as B_FILL => BFill(buffer::BFill),
        /// `/b_gen`: a fill command writes a buffer.
        "/b_gen" as B_GEN => BGen(buffer::BGen),
        /// `/b_close`: closes the sound file of a buffer.
        "/b_close" as B_CLOSE => BClose(buffer::BClose),
        /// `/b_query`: the server replies `/b_info` for buffers.
        "/b_query" as B_QUERY => BQuery(buffer::BQuery),
        /// `/b_get`: the server replies `/b_set` with values of samples.
        "/b_get" as B_GET => BGet(buffer::BGet),
        /// `/b_getn`: the server replies `/b_setn` with values of ranges of
        /// samples.
        "/b_getn" as B_GETN => BGetn(buffer::BGetn),
        /// `/c_set`: sets control buses.
        "/c_set" as C_SET => CSet(control_bus::CSet),
        /// `/c_setn`: sets ranges of control buses.
        "/c_setn" as C_SETN => CSetn(control_bus::CSetn),
        /// `/c_fill`: sets ranges of control buses to one value each.
        "/c_fill" as C_FILL => CFill(control_bus::CFill),
        /// `/c_get`: the server replies `/c_set` with values of buses.
        "/c_get" as C_GET => CGet(control_bus::CGet),
        /// `/c_getn`: the server replies `/c_setn` with values of ranges of
        /// buses.
        "/c_getn" as C_GETN => CGetn(control_bus::CGetn),
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
    /// Commands that run together, at the time tag.
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

pub(crate) fn write_completion(writer: &mut Writer, completion: Option<&Completion>) {
    if let Some(completion) = completion {
        writer.blob(completion.to_bytes());
    }
}

/// Reads the completion message at the reader's position, when there is
/// one. With `placeholder`, an int 0 there stands for none.
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
