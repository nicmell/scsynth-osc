//! Replies to commands: `/done` and `/fail`. The Server Command Reference
//! also lists `/late`, but scsynth 3.14.1 never sends it.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/done`: which asynchronous command completed. scsynth
/// sends it to the sender of the command, once the completion message of
/// the command ran.
///
/// Some failures still end with `/done`: `/d_recv` of invalid bytes, and
/// the reads of a buffer whose start frame is past its end. A failed
/// allocation of `/b_alloc` ends with no reply at all.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Done {
    /// `/done /quit`.
    Quit,
    /// `/done /notify clientID maxLogins`: the sender is registered.
    Notify {
        /// The ID of the sender among the clients of the server.
        client_id: i32,
        /// How many clients the server takes at once.
        max_logins: i32,
    },
    /// `/done /notify`: the registration of the sender ended.
    NotifyOff,
    /// `/done /d_recv`.
    DRecv,
    /// `/done /d_load`.
    DLoad,
    /// `/done /d_loadDir`.
    DLoadDir,
    /// `/done <command> bufNum`: a buffer command completed.
    Buffer {
        /// The command.
        command: BufferCommand,
        /// The buffer number, as the command gave it.
        buffer: i32,
    },
    /// `/done <name>`: an asynchronous command of a plugin completed. The
    /// name is the plugin's choice, with or without a leading slash.
    Plugin {
        /// The name that the plugin gave.
        name: String,
    },
}

/// The buffer commands whose `/done` names the buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BufferCommand {
    /// `/b_alloc`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_alloc"))]
    BAlloc,
    /// `/b_allocRead`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_allocRead"))]
    BAllocRead,
    /// `/b_allocReadChannel`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_allocReadChannel"))]
    BAllocReadChannel,
    /// `/b_read`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_read"))]
    BRead,
    /// `/b_readChannel`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_readChannel"))]
    BReadChannel,
    /// `/b_write`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_write"))]
    BWrite,
    /// `/b_free`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_free"))]
    BFree,
    /// `/b_zero`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_zero"))]
    BZero,
    /// `/b_gen`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_gen"))]
    BGen,
    /// `/b_close`.
    #[cfg_attr(feature = "serde", serde(rename = "/b_close"))]
    BClose,
}

impl BufferCommand {
    const ALL: [Self; 10] = [
        Self::BAlloc,
        Self::BAllocRead,
        Self::BAllocReadChannel,
        Self::BRead,
        Self::BReadChannel,
        Self::BWrite,
        Self::BFree,
        Self::BZero,
        Self::BGen,
        Self::BClose,
    ];

    /// The address of the command.
    pub fn address(self) -> &'static str {
        match self {
            Self::BAlloc => "/b_alloc",
            Self::BAllocRead => "/b_allocRead",
            Self::BAllocReadChannel => "/b_allocReadChannel",
            Self::BRead => "/b_read",
            Self::BReadChannel => "/b_readChannel",
            Self::BWrite => "/b_write",
            Self::BFree => "/b_free",
            Self::BZero => "/b_zero",
            Self::BGen => "/b_gen",
            Self::BClose => "/b_close",
        }
    }

    fn from_address(address: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|command| command.address() == address)
    }
}

impl Payload for Done {
    fn write(&self, writer: &mut Writer) {
        match self {
            Self::Quit => writer.string("/quit"),
            Self::Notify {
                client_id,
                max_logins,
            } => {
                writer.string("/notify");
                writer.int(*client_id);
                writer.int(*max_logins);
            }
            Self::NotifyOff => writer.string("/notify"),
            Self::DRecv => writer.string("/d_recv"),
            Self::DLoad => writer.string("/d_load"),
            Self::DLoadDir => writer.string("/d_loadDir"),
            Self::Buffer { command, buffer } => {
                writer.string(command.address());
                writer.int(*buffer);
            }
            Self::Plugin { name } => writer.string(name),
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let name = reader.string()?;
        Ok(match name.as_str() {
            "/quit" => Self::Quit,
            "/notify" if reader.remains() => Self::Notify {
                client_id: reader.int()?,
                max_logins: reader.int()?,
            },
            "/notify" => Self::NotifyOff,
            "/d_recv" => Self::DRecv,
            "/d_load" => Self::DLoad,
            "/d_loadDir" => Self::DLoadDir,
            other => match BufferCommand::from_address(other) {
                Some(command) => Self::Buffer {
                    command,
                    buffer: reader.int()?,
                },
                None => Self::Plugin { name },
            },
        })
    }
}

/// The arguments of `/fail`: a command failed. scsynth sends it to the
/// sender of the command.
///
/// The failures that a command returns depend on the mode of `/error`.
/// `Command not found`, the failures of `/notify` and the failures of the
/// background stages of asynchronous commands are always sent.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fail {
    /// The command, as scsynth names it: its address, or the name without
    /// the slash when the sender used that alias.
    pub command: String,
    /// The error, verbatim. The messages of the background stages, such as
    /// `File '…' could not be opened: …`, end with a newline.
    pub message: String,
    /// The buffer number, for a failure of an asynchronous buffer command;
    /// the existing client ID, for `/notify` from a registered sender.
    pub value: Option<i32>,
}

impl Payload for Fail {
    fn write(&self, writer: &mut Writer) {
        writer.string(&self.command);
        writer.string(&self.message);
        if let Some(value) = self.value {
            writer.int(value);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            command: reader.string()?,
            message: reader.string()?,
            value: reader.optional_int()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Int, String as Text};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn every_done_has_the_arguments_of_its_command() {
        assert_wire(&Reply::Done(Done::Quit), "/done", vec![text("/quit")]);
        let registered = Done::Notify {
            client_id: 0,
            max_logins: 32,
        };
        let wire = vec![text("/notify"), Int(0), Int(32)];
        assert_wire(&Reply::Done(registered), "/done", wire);
        assert_wire(
            &Reply::Done(Done::NotifyOff),
            "/done",
            vec![text("/notify")],
        );
        assert_wire(&Reply::Done(Done::DRecv), "/done", vec![text("/d_recv")]);
        assert_wire(&Reply::Done(Done::DLoad), "/done", vec![text("/d_load")]);
        assert_wire(
            &Reply::Done(Done::DLoadDir),
            "/done",
            vec![text("/d_loadDir")],
        );
        for command in BufferCommand::ALL {
            let done = Done::Buffer { command, buffer: 4 };
            let wire = vec![text(command.address()), Int(4)];
            assert_wire(&Reply::Done(done), "/done", wire);
        }
        let plugin = Done::Plugin {
            name: "cmdDemoFunc".to_owned(),
        };
        assert_wire(&Reply::Done(plugin), "/done", vec![text("cmdDemoFunc")]);
    }

    #[test]
    fn a_done_of_a_buffer_command_names_its_buffer() {
        assert_eq!(
            decode_error::<Reply>("/done", vec![text("/b_alloc")]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "an int"
            }
        );
        assert_eq!(
            decode_error::<Reply>("/done", vec![text("/quit"), Int(1)]),
            DecodeErrorKind::Trailing { index: 1 }
        );
    }

    #[test]
    fn a_fail_keeps_its_message_verbatim() {
        let read = Fail {
            command: "/b_read".to_owned(),
            message: "File 'a.wav' could not be opened: System error\n".to_owned(),
            value: Some(2),
        };
        let wire = vec![
            text("/b_read"),
            text("File 'a.wav' could not be opened: System error\n"),
            Int(2),
        ];
        assert_wire(&Reply::Fail(read), "/fail", wire);
        let duplicate = Fail {
            command: "/s_new".to_owned(),
            message: "duplicate node ID".to_owned(),
            value: None,
        };
        let wire = vec![text("/s_new"), text("duplicate node ID")];
        assert_wire(&Reply::Fail(duplicate), "/fail", wire);
    }
}
