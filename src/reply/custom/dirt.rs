//! The answers of sc-app3's pump about the Dirt of an element of a plugin:
//! the outcome of each start and of each load of samples, and the sounds
//! that the pump drops.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_id, read_sample, read_uuid, write_uuid};
use crate::error::DecodeError;

/// The arguments of `/loadSample.success`: see
/// [`crate::command::custom::Sample`].
pub type Sample = crate::command::custom::Sample;

/// The arguments of `/loadSample.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleFailure {
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element. It cannot be empty.
    pub id: String,
    /// The name of the samples: 1 to 255 bytes.
    pub name: String,
    /// Why the load failed.
    pub reason: String,
}

impl Payload for SampleFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        writer.string(&self.name);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            name: read_sample(reader)?,
            reason: reader.string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::String as Text;

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::reply::custom::{Node, NodeFailure};
    use crate::testing::{assert_wire, decode_error};

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const ELEMENT: &str = "5f0a3c9e21d47";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_a_start_and_of_a_drop_have_their_wire_form() {
        let started = Node {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
        };
        assert_wire(
            &Reply::LoadDirtSuccess(started),
            "/loadDirt.success",
            vec![text(PLUGIN), text(ELEMENT)],
        );
        let refused = NodeFailure {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            reason: "No bus is left for the Dirt.".to_owned(),
        };
        assert_wire(
            &Reply::LoadDirtFailure(refused),
            "/loadDirt.failure",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("No bus is left for the Dirt."),
            ],
        );
        let dropped = NodeFailure {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            reason: "The sound came too late.".to_owned(),
        };
        assert_wire(
            &Reply::DirtPlayFailure(dropped),
            "/dirt.play.failure",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("The sound came too late."),
            ],
        );
    }

    #[test]
    fn the_answers_of_a_load_of_samples_have_their_wire_form() {
        let sample = Sample {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            name: "bd".to_owned(),
        };
        assert_wire(
            &Reply::LoadSampleSuccess(sample),
            "/loadSample.success",
            vec![text(PLUGIN), text(ELEMENT), text("bd")],
        );
        let failure = SampleFailure {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            name: "bd".to_owned(),
            reason: "No samples of that name.".to_owned(),
        };
        assert_wire(
            &Reply::LoadSampleFailure(failure),
            "/loadSample.failure",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("bd"),
                text("No samples of that name."),
            ],
        );
    }

    #[test]
    fn a_failure_of_a_load_of_samples_names_its_samples() {
        let failure = |name: &str| vec![text(PLUGIN), text("a"), text(name), text("why")];
        for name in [String::new(), "n".repeat(256)] {
            let kind = decode_error::<Reply>("/loadSample.failure", failure(&name));
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
                "{}: {kind:?}",
                name.len()
            );
        }
        let kind = decode_error::<Reply>("/loadSample.success", vec![text(PLUGIN), text("a")]);
        assert_eq!(
            kind,
            DecodeErrorKind::Missing {
                index: 2,
                expected: "a string"
            }
        );
    }
}
