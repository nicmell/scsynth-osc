//! The commands that a client sends to scsynth, in the sections of the
//! Server Command Reference.

pub mod synth_def;
pub mod top_level;

use rosc::{OscBundle, OscPacket, OscType};

use crate::codec::{Message, Reader, Writer, messages};
use crate::error::{DecodeError, DecodeErrorKind};
use crate::packet::{Timetag, decode_osc, encode_osc};

messages! {
    /// A command for scsynth: one variant per address of the Server Command
    /// Reference.
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
    }
}

/// What an asynchronous command runs once its work is done, before the
/// server replies `/done`. It travels as a blob in the command, and its
/// replies go to the sender of the command.
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
                timetag: (*time).into(),
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
                time: bundle.timetag.into(),
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
