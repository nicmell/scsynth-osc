//! The answers of sc-app3's pump about the group of each plugin of a
//! session.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_uuid, write_uuid};
use crate::error::DecodeError;

/// The arguments of `/loadPlugin.success` and `/unloadPlugin.success`: see
/// [`crate::command::custom::Plugin`].
pub type Plugin = crate::command::custom::Plugin;

/// The arguments of `/loadPlugin.failure` and `/unloadPlugin.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PluginFailure {
    /// The plugin.
    pub plugin: Uuid,
    /// Why the command failed.
    pub reason: String,
}

impl Payload for PluginFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
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

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_the_pump_have_their_wire_form() {
        let plugin = Plugin {
            plugin: uuid(PLUGIN),
        };
        let wire = || vec![text(PLUGIN)];
        assert_wire(
            &Reply::LoadPluginSuccess(plugin),
            "/loadPlugin.success",
            wire(),
        );
        assert_wire(
            &Reply::UnloadPluginSuccess(plugin),
            "/unloadPlugin.success",
            wire(),
        );
        let failure = PluginFailure {
            plugin: uuid(PLUGIN),
            reason: "The root group is not ready.".to_owned(),
        };
        let wire = || vec![text(PLUGIN), text("The root group is not ready.")];
        assert_wire(
            &Reply::LoadPluginFailure(failure.clone()),
            "/loadPlugin.failure",
            wire(),
        );
        assert_wire(
            &Reply::UnloadPluginFailure(failure),
            "/unloadPlugin.failure",
            wire(),
        );
    }

    #[test]
    fn a_failure_names_its_reason() {
        assert_eq!(
            decode_error::<Reply>("/loadPlugin.failure", vec![text(PLUGIN)]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "a string"
            }
        );
        let kind = decode_error::<Reply>("/loadPlugin.success", vec![text("x")]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
            "{kind:?}"
        );
    }
}
