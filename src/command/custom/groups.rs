//! The commands of sc-app3's pump about the groups of a session: its root
//! group, and the group of each of its plugins.

use uuid::Uuid;

use super::{read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/loadRoot` and `/unloadRoot`, and of their
/// `.success`: the session whose root group the pump loads or frees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Root {
    /// The session.
    pub session: Uuid,
}

impl Payload for Root {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
        })
    }
}

/// The arguments of `/loadPlugin` and `/unloadPlugin`, and of their
/// `.success`: a plugin of a session, whose group the pump loads or frees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plugin {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
}

impl Payload for Plugin {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
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

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_commands_of_the_pump_have_their_wire_form() {
        let root = Root {
            session: uuid(SESSION),
        };
        assert_wire(&Command::LoadRoot(root), "/loadRoot", vec![text(SESSION)]);
        assert_wire(
            &Command::UnloadRoot(root),
            "/unloadRoot",
            vec![text(SESSION)],
        );
        let plugin = Plugin {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
        };
        let wire = vec![text(SESSION), text(PLUGIN)];
        assert_wire(&Command::LoadPlugin(plugin), "/loadPlugin", wire.clone());
        assert_wire(&Command::UnloadPlugin(plugin), "/unloadPlugin", wire);
    }

    #[test]
    fn the_addresses_of_the_pump_are_those_of_its_commands() {
        let root = Root {
            session: uuid(SESSION),
        };
        let plugin = Plugin {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
        };
        let commands = [
            Command::LoadRoot(root),
            Command::UnloadRoot(root),
            Command::LoadPlugin(plugin),
            Command::UnloadPlugin(plugin),
        ];
        assert_eq!(
            commands.map(|command| command.address()),
            crate::command::custom::ADDRESSES
        );
    }

    #[test]
    fn a_uuid_is_lowercase_with_hyphens() {
        for other in [
            "not a uuid",
            "5049297F-342B-4B8B-8C62-54EDBB5D3335",
            "5049297f342b4b8b8c6254edbb5d3335",
            "{5049297f-342b-4b8b-8c62-54edbb5d3335}",
            "urn:uuid:5049297f-342b-4b8b-8c62-54edbb5d3335",
        ] {
            let kind = decode_error::<Command>("/loadRoot", vec![text(other)]);
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
                "{other}: {kind:?}"
            );
        }
        assert_eq!(
            decode_error::<Command>("/loadPlugin", vec![text(SESSION), Int(1)]),
            DecodeErrorKind::WrongType {
                index: 1,
                expected: "a string",
                found: 'i'
            }
        );
    }
}
