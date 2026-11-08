//! The commands of sc-app3's pump about the synth definitions of a page:
//! the pump loads each one into scsynth for a plugin of the session, and
//! frees it.

use uuid::Uuid;

use super::{read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The most bytes in the name of a synth definition. A synth definition
/// file gives the length of the name in one byte, and scsynth refuses 256
/// or more (`SC_GraphDef.cpp:75-81`).
const MAX_NAME: usize = 255;

/// The arguments of `/loadSynthdef`: a synth definition that the pump loads
/// into scsynth for a plugin of a session.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadSynthdef {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The name of the definition, as `data` holds it: 1 to 255 bytes.
    pub name: String,
    /// The bytes of a synth definition file. They cannot be empty.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub data: Vec<u8>,
}

impl Payload for LoadSynthdef {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.name);
        writer.blob(self.data.clone());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let session = read_uuid(reader)?;
        let plugin = read_uuid(reader)?;
        let name = read_name(reader)?;
        let index = reader.index();
        let data = reader.blob()?;
        if data.is_empty() {
            return Err(reader.invalid(index, "empty synth definition data"));
        }
        Ok(Self {
            session,
            plugin,
            name,
            data,
        })
    }
}

/// The arguments of `/unloadSynthdef`, and of the `.success` of both
/// commands: a synth definition of a plugin of a session.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Synthdef {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The name of the definition: 1 to 255 bytes.
    pub name: String,
}

impl Payload for Synthdef {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.name);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            name: read_name(reader)?,
        })
    }
}

/// The name of a synth definition: 1 to 255 bytes.
pub(crate) fn read_name(reader: &mut Reader<'_>) -> Result<String, DecodeError> {
    let index = reader.index();
    let name = reader.string()?;
    if name.is_empty() || name.len() > MAX_NAME {
        return Err(reader.invalid(
            index,
            format!(
                "a name of {} bytes, where a synth definition takes 1 to {MAX_NAME}",
                name.len()
            ),
        ));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Blob, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode, decode_error};

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_commands_of_the_synthdefs_have_their_wire_form() {
        let load = LoadSynthdef {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            name: "a1b2/fm-sine".to_owned(),
            data: vec![83, 67, 103, 102],
        };
        assert_wire(
            &Command::LoadSynthdef(load),
            "/loadSynthdef",
            vec![
                text(SESSION),
                text(PLUGIN),
                text("a1b2/fm-sine"),
                Blob(vec![83, 67, 103, 102]),
            ],
        );
        let unload = Synthdef {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            name: "a1b2/fm-sine".to_owned(),
        };
        assert_wire(
            &Command::UnloadSynthdef(unload),
            "/unloadSynthdef",
            vec![text(SESSION), text(PLUGIN), text("a1b2/fm-sine")],
        );
    }

    #[test]
    fn a_name_takes_1_to_255_bytes_and_the_data_is_not_empty() {
        let unload = |name: &str| vec![text(SESSION), text(PLUGIN), text(name)];
        for name in [String::new(), "n".repeat(256)] {
            let kind = decode_error::<Command>("/unloadSynthdef", unload(&name));
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
                "{}: {kind:?}",
                name.len()
            );
        }
        let longest = "n".repeat(255);
        assert_eq!(
            decode::<Command>("/unloadSynthdef", unload(&longest)),
            Command::UnloadSynthdef(Synthdef {
                session: uuid(SESSION),
                plugin: uuid(PLUGIN),
                name: longest.clone(),
            })
        );
        let kind = decode_error::<Command>(
            "/loadSynthdef",
            vec![text(SESSION), text(PLUGIN), text("a1b2/x"), Blob(vec![])],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 3, .. }),
            "{kind:?}"
        );
    }
}
