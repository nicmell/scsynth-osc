//! Synth definition commands.

use super::{Completion, read_completion, write_completion};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/d_recv`. The server replies `/done /d_recv`, even
/// when the bytes hold no valid synth definition: it then defines nothing.
/// It sends `/d_removed` to registered clients for each unused definition
/// that a new one replaces.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DRecv {
    /// The bytes of a synth definition file. They cannot be empty: scsynth
    /// refuses an empty blob.
    pub data: Vec<u8>,
    /// What runs once the definitions are in place.
    pub completion: Option<Completion>,
}

impl Payload for DRecv {
    fn write(&self, writer: &mut Writer) {
        writer.blob(self.data.clone());
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let index = reader.index();
        let data = reader.blob()?;
        if data.is_empty() {
            return Err(reader.invalid(index, "empty synth definition data"));
        }
        Ok(Self {
            data,
            completion: read_completion(reader, true)?,
        })
    }
}

/// The arguments of `/d_load`. scsynth loads the files whose names end in
/// `.scsyndef`. It replies `/done /d_load`, or `/fail /d_load <path>`, the
/// path as the whole message, when it loads no definition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DLoad {
    /// The path of a file, or a pattern such as `synthdefs/perc-*`.
    pub path: String,
    /// What runs once the definitions are in place.
    pub completion: Option<Completion>,
}

impl Payload for DLoad {
    fn write(&self, writer: &mut Writer) {
        writer.string(&self.path);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            path: reader.string()?,
            completion: read_completion(reader, true)?,
        })
    }
}

/// The arguments of `/d_loadDir`. scsynth loads the `.scsyndef` files of
/// the directory and of its subdirectories. It replies `/done /d_loadDir`,
/// or `/fail /d_loadDir <path>` when the directory is missing or holds no
/// definition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DLoadDir {
    /// The path of the directory.
    pub path: String,
    /// What runs once the definitions are in place.
    pub completion: Option<Completion>,
}

impl Payload for DLoadDir {
    fn write(&self, writer: &mut Writer) {
        writer.string(&self.path);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            path: reader.string()?,
            completion: read_completion(reader, true)?,
        })
    }
}

/// The arguments of `/d_free`. The definitions go at once; the synths that
/// use them keep running. scsynth ignores unknown names, and sends no reply.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DFree {
    /// The names of the definitions.
    pub names: Vec<String>,
}

impl Payload for DFree {
    fn write(&self, writer: &mut Writer) {
        for name in &self.names {
            writer.string(name);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            names: reader.rest(Reader::string)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Blob, Int, String as Text};
    use rosc::{OscBundle, OscPacket};

    use super::*;
    use crate::Command;
    use crate::command::top_level::SyncId;
    use crate::error::DecodeErrorKind;
    use crate::packet::Timetag;
    use crate::testing::{assert_wire, bytes, decode_error};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_synth_definition_commands_have_their_wire_form() {
        let recv = DRecv {
            data: vec![1, 2, 3],
            completion: None,
        };
        assert_wire(&Command::DRecv(recv), "/d_recv", vec![Blob(vec![1, 2, 3])]);
        let load = DLoad {
            path: "defs/perc-*".to_owned(),
            completion: None,
        };
        assert_wire(&Command::DLoad(load), "/d_load", vec![text("defs/perc-*")]);
        let dir = DLoadDir {
            path: "defs".to_owned(),
            completion: None,
        };
        assert_wire(&Command::DLoadDir(dir), "/d_loadDir", vec![text("defs")]);
        let free = DFree {
            names: vec!["a".to_owned(), "b".to_owned()],
        };
        assert_wire(&Command::DFree(free), "/d_free", vec![text("a"), text("b")]);
        assert_wire(&Command::DFree(DFree { names: vec![] }), "/d_free", vec![]);
    }

    #[test]
    fn a_completion_message_travels_as_the_bytes_of_its_command() {
        let load = DLoad {
            path: "a.scsyndef".to_owned(),
            completion: Some(Completion::Message(Box::new(Command::Sync(SyncId(1))))),
        };
        let completion = Blob(bytes("/sync", vec![Int(1)]));
        assert_wire(
            &Command::DLoad(load),
            "/d_load",
            vec![text("a.scsyndef"), completion],
        );
    }

    #[test]
    fn a_completion_bundle_carries_its_commands_in_order() {
        let recv = Command::DRecv(DRecv {
            data: vec![9],
            completion: Some(Completion::Bundle {
                time: Timetag::IMMEDIATELY,
                commands: vec![Command::Status, Command::Sync(SyncId(2))],
            }),
        });
        assert_eq!(Command::decode(&recv.encode()), Ok(recv));
    }

    #[test]
    fn the_placeholder_of_sclang_stands_for_no_completion_message() {
        let bytes = bytes("/d_load", vec![text("a"), Int(0)]);
        let load = DLoad {
            path: "a".to_owned(),
            completion: None,
        };
        assert_eq!(Command::decode(&bytes), Ok(Command::DLoad(load)));
        let kind = decode_error::<Command>("/d_load", vec![text("a"), Int(5)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn nothing_follows_a_completion_message() {
        let completion = Blob(bytes("/status", vec![]));
        assert_eq!(
            decode_error::<Command>("/d_load", vec![text("a"), completion, Int(1)]),
            DecodeErrorKind::Trailing { index: 2 }
        );
    }

    #[test]
    fn an_empty_blob_is_refused() {
        for (address, args, index) in [
            ("/d_recv", vec![Blob(vec![])], 0),
            ("/d_load", vec![text("a"), Blob(vec![])], 1),
        ] {
            let kind = decode_error::<Command>(address, args);
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: at, .. } if at == index),
                "{address}: {kind:?}"
            );
        }
    }

    #[test]
    fn a_completion_message_holds_commands_and_no_nested_bundle() {
        let unknown = Blob(bytes("/beat", vec![]));
        let nested = rosc::encoder::encode(&OscPacket::Bundle(OscBundle {
            timetag: Timetag::IMMEDIATELY.to_osc(),
            content: vec![OscPacket::Bundle(OscBundle {
                timetag: Timetag::IMMEDIATELY.to_osc(),
                content: vec![],
            })],
        }))
        .expect("a bundle encodes");
        for completion in [unknown, Blob(nested)] {
            let kind = decode_error::<Command>("/d_load", vec![text("a"), completion]);
            assert!(
                matches!(kind, DecodeErrorKind::Completion { index: 1, .. }),
                "{kind:?}"
            );
        }
    }
}
