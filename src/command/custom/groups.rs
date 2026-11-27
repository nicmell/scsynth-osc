//! The commands of sc-app3's pump about the group of each plugin of a
//! session.

use uuid::Uuid;

use super::{read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/loadPlugin` and `/unloadPlugin`, and of their
/// `.success`: a plugin of the session, whose group the pump loads or
/// frees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plugin {
    /// The plugin.
    pub plugin: Uuid,
}

impl Payload for Plugin {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_commands_of_the_pump_have_their_wire_form() {
        let plugin = Plugin {
            plugin: uuid(PLUGIN),
        };
        let wire = vec![text(PLUGIN)];
        assert_wire(&Command::LoadPlugin(plugin), "/loadPlugin", wire.clone());
        assert_wire(&Command::UnloadPlugin(plugin), "/unloadPlugin", wire);
    }

    #[test]
    fn a_uuid_is_lowercase_with_hyphens() {
        for other in [
            "not a uuid",
            "0B4E1A2C-9D7F-4E3A-8B6C-1F2E3D4C5B6A",
            "0b4e1a2c9d7f4e3a8b6c1f2e3d4c5b6a",
            "{0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a}",
            "urn:uuid:0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a",
        ] {
            let kind = decode_error::<Command>("/loadPlugin", vec![text(other)]);
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
                "{other}: {kind:?}"
            );
        }
        assert_eq!(
            decode_error::<Command>("/loadPlugin", vec![Int(1)]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "a string",
                found: 'i'
            }
        );
    }
}
