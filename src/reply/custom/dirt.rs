//! The answers of sc-app3's pump about the Dirt of an element of a plugin,
//! by the node of the group of the element: the outcome of each start, and
//! the sounds that the pump drops.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_uuid, write_uuid};
use crate::error::DecodeError;
use crate::types::NodeId;

/// The arguments of `/loadDirt.success`: the Dirt in the group `node` of
/// an element of a plugin runs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dirt {
    /// The plugin.
    pub plugin: Uuid,
    /// The node of the group of the element.
    pub node: NodeId,
}

impl Payload for Dirt {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        self.node.write(writer);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            node: NodeId::read(reader)?,
        })
    }
}

/// The arguments of `/loadDirt.failure` and `/dirt.play.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirtFailure {
    /// The plugin.
    pub plugin: Uuid,
    /// The node of the group of the element.
    pub node: NodeId,
    /// Why the command failed.
    pub reason: String,
}

impl Payload for DirtFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        self.node.write(writer);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            node: NodeId::read(reader)?,
            reason: reader.string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Int, String as Text};
    use uuid::Uuid;

    use super::{Dirt, DirtFailure};
    use crate::Reply;
    use crate::testing::assert_wire;
    use crate::types::NodeId;

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const NODE: i32 = 67_240_961;

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_a_start_and_of_a_drop_have_their_wire_form() {
        let started = Dirt {
            plugin: uuid(PLUGIN),
            node: NodeId(NODE),
        };
        assert_wire(
            &Reply::LoadDirtSuccess(started),
            "/loadDirt.success",
            vec![text(PLUGIN), Int(NODE)],
        );
        let refused = DirtFailure {
            plugin: uuid(PLUGIN),
            node: NodeId(NODE),
            reason: "No bus is left for the Dirt.".to_owned(),
        };
        assert_wire(
            &Reply::LoadDirtFailure(refused),
            "/loadDirt.failure",
            vec![
                text(PLUGIN),
                Int(NODE),
                text("No bus is left for the Dirt."),
            ],
        );
        let dropped = DirtFailure {
            plugin: uuid(PLUGIN),
            node: NodeId(NODE),
            reason: "The sound came too late.".to_owned(),
        };
        assert_wire(
            &Reply::DirtPlayFailure(dropped),
            "/dirt.play.failure",
            vec![text(PLUGIN), Int(NODE), text("The sound came too late.")],
        );
    }
}
