//! The command of sc-app3's pump about the states of the elements of a
//! plugin: the value that a page writes, which the pump keeps.

use uuid::Uuid;

use super::{read_id, read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/setState`: the value of a state of an element of a
/// plugin, as a page wrote it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetState {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element of the state. It cannot be empty.
    pub key: String,
    /// The value, as the text of a JSON value. The crate does not read it.
    pub value: String,
}

impl Payload for SetState {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.key);
        writer.string(&self.value);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            key: read_id(reader)?,
            value: reader.string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::String as Text;

    use super::*;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};
    use crate::{Command, Reply};

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_messages_of_the_states_have_their_wire_form() {
        let set = SetState {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            key: "c37dbff1fc4849b22e0d54edbe2fa1c6".to_owned(),
            value: "[0.5, 2]".to_owned(),
        };
        assert_wire(
            &Command::SetState(set),
            "/setState",
            vec![
                text(SESSION),
                text(PLUGIN),
                text("c37dbff1fc4849b22e0d54edbe2fa1c6"),
                text("[0.5, 2]"),
            ],
        );
        let change = crate::reply::custom::Plugin {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
        };
        assert_wire(
            &Reply::StateChange(change),
            "/stateChange",
            vec![text(SESSION), text(PLUGIN)],
        );
    }

    #[test]
    fn a_key_is_not_empty() {
        let kind = decode_error::<Command>(
            "/setState",
            vec![text(SESSION), text(PLUGIN), text(""), text("1")],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
    }
}
