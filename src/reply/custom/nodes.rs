//! The answers of sc-app3's pump about the nodes of the elements of a
//! plugin: the outcome of each load, with the node of a success, and of
//! each unload.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_id, read_uuid, write_uuid};
use crate::error::DecodeError;
use crate::types::NodeId;

/// The arguments of `/loadGroup.success` and `/loadSynth.success`: the node
/// of an element of a plugin in scsynth.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeLoaded {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element. It cannot be empty.
    pub id: String,
    /// The node of the element.
    pub node: NodeId,
}

impl Payload for NodeLoaded {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        self.node.write(writer);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            node: NodeId::read(reader)?,
        })
    }
}

/// The arguments of `/unloadGroup.success` and `/unloadSynth.success`: see
/// [`crate::command::custom::Node`].
pub type Node = crate::command::custom::Node;

/// The arguments of `/loadGroup.failure`, `/loadSynth.failure`,
/// `/unloadGroup.failure` and `/unloadSynth.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeFailure {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element. It cannot be empty.
    pub id: String,
    /// Why the load or the unload failed.
    pub reason: String,
}

impl Payload for NodeFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            reason: reader.string()?,
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

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_the_nodes_have_their_wire_form() {
        let loaded = NodeLoaded {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            id: "1bcdb1b42939e3".to_owned(),
            node: NodeId(67_174_405),
        };
        let wire = vec![
            text(SESSION),
            text(PLUGIN),
            text("1bcdb1b42939e3"),
            Int(67_174_405),
        ];
        assert_wire(
            &Reply::LoadGroupSuccess(loaded.clone()),
            "/loadGroup.success",
            wire.clone(),
        );
        assert_wire(&Reply::LoadSynthSuccess(loaded), "/loadSynth.success", wire);
        let failure = NodeFailure {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            id: "1bcdb1b42939e3".to_owned(),
            reason: "The group of the plugin is not ready.".to_owned(),
        };
        let wire = vec![
            text(SESSION),
            text(PLUGIN),
            text("1bcdb1b42939e3"),
            text("The group of the plugin is not ready."),
        ];
        assert_wire(
            &Reply::LoadGroupFailure(failure.clone()),
            "/loadGroup.failure",
            wire.clone(),
        );
        assert_wire(
            &Reply::LoadSynthFailure(failure.clone()),
            "/loadSynth.failure",
            wire.clone(),
        );
        assert_wire(
            &Reply::UnloadGroupFailure(failure.clone()),
            "/unloadGroup.failure",
            wire.clone(),
        );
        assert_wire(
            &Reply::UnloadSynthFailure(failure),
            "/unloadSynth.failure",
            wire,
        );
        let freed = Node {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            id: "1bcdb1b42939e3".to_owned(),
        };
        let wire = vec![text(SESSION), text(PLUGIN), text("1bcdb1b42939e3")];
        assert_wire(
            &Reply::UnloadGroupSuccess(freed.clone()),
            "/unloadGroup.success",
            wire.clone(),
        );
        assert_wire(
            &Reply::UnloadSynthSuccess(freed),
            "/unloadSynth.success",
            wire,
        );
    }

    #[test]
    fn an_answer_with_an_empty_id_does_not_decode() {
        let kind = decode_error::<Reply>(
            "/loadSynth.success",
            vec![text(SESSION), text(PLUGIN), text(""), Int(7)],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
    }
}
