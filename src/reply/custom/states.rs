//! The answers of sc-app3's pump about the states of the elements of a
//! plugin: the values that a page reads.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_uuid, write_uuid};
use crate::error::DecodeError;

/// The arguments of `/getState.success`: the values of the states of the
/// elements of a plugin, as the pump keeps them.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct States {
    /// The plugin.
    pub plugin: Uuid,
    /// The values, as the text of a JSON object from the id of the element
    /// of each state to its value. The crate does not read it.
    pub states: String,
}

impl Payload for States {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.string(&self.states);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            states: reader.string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::String as Text;

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::reply::custom::PluginFailure;
    use crate::testing::{assert_wire, decode_error};

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const STATES: &str = r#"{"c37dbff1fc4849b22e0d54edbe2fa1c6":0.8}"#;

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_a_read_have_their_wire_form() {
        let states = States {
            plugin: uuid(PLUGIN),
            states: STATES.to_owned(),
        };
        assert_wire(
            &Reply::GetStateSuccess(states),
            "/getState.success",
            vec![text(PLUGIN), text(STATES)],
        );
        let failure = PluginFailure {
            plugin: uuid(PLUGIN),
            reason: "No such plugin.".to_owned(),
        };
        assert_wire(
            &Reply::GetStateFailure(failure),
            "/getState.failure",
            vec![text(PLUGIN), text("No such plugin.")],
        );
    }

    #[test]
    fn a_success_carries_the_states() {
        assert_eq!(
            decode_error::<Reply>("/getState.success", vec![text(PLUGIN)]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "a string"
            }
        );
    }
}
