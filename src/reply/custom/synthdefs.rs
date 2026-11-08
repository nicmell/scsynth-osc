//! The answers of sc-app3's pump about the synth definitions of a page:
//! the outcome of each load and of each free.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_name, read_uuid, write_uuid};
use crate::error::DecodeError;

/// The arguments of `/loadSynthdef.success` and `/unloadSynthdef.success`:
/// see [`crate::command::custom::Synthdef`].
pub type Synthdef = crate::command::custom::Synthdef;

/// The arguments of `/loadSynthdef.failure` and `/unloadSynthdef.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SynthdefFailure {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The name of the definition: 1 to 255 bytes.
    pub name: String,
    /// Why the command failed.
    pub reason: String,
}

impl Payload for SynthdefFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.name);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            name: read_name(reader)?,
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
    fn the_answers_of_the_synthdefs_have_their_wire_form() {
        let synthdef = Synthdef {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            name: "a1b2/fm-sine".to_owned(),
        };
        let wire = vec![text(SESSION), text(PLUGIN), text("a1b2/fm-sine")];
        assert_wire(
            &Reply::LoadSynthdefSuccess(synthdef.clone()),
            "/loadSynthdef.success",
            wire.clone(),
        );
        assert_wire(
            &Reply::UnloadSynthdefSuccess(synthdef),
            "/unloadSynthdef.success",
            wire,
        );
        let failure = SynthdefFailure {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            name: "a1b2/fm-sine".to_owned(),
            reason: "The definition is too large to send to scsynth.".to_owned(),
        };
        let wire = vec![
            text(SESSION),
            text(PLUGIN),
            text("a1b2/fm-sine"),
            text("The definition is too large to send to scsynth."),
        ];
        assert_wire(
            &Reply::LoadSynthdefFailure(failure.clone()),
            "/loadSynthdef.failure",
            wire.clone(),
        );
        assert_wire(
            &Reply::UnloadSynthdefFailure(failure),
            "/unloadSynthdef.failure",
            wire,
        );
    }

    #[test]
    fn a_failure_names_its_definition() {
        let kind = decode_error::<Reply>(
            "/loadSynthdef.failure",
            vec![text(SESSION), text(PLUGIN), text(""), text("why")],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
    }
}
