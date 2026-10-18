//! The answers of sc-app3's pump, which scsynth never sends. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::pump::{read_uuid, write_uuid};
use crate::error::DecodeError;

/// The arguments of `/loadRoot.success` and `/unloadRoot.success`: see
/// [`crate::command::pump::Root`].
pub type Root = crate::command::pump::Root;

/// The arguments of `/loadPlugin.success` and `/unloadPlugin.success`: see
/// [`crate::command::pump::Plugin`].
pub type Plugin = crate::command::pump::Plugin;

/// The arguments of `/loadRoot.failure` and `/unloadRoot.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RootFailure {
    /// The session.
    pub session: Uuid,
    /// Why the command failed.
    pub reason: String,
}

impl Payload for RootFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            reason: reader.string()?,
        })
    }
}

/// The arguments of `/loadPlugin.failure` and `/unloadPlugin.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PluginFailure {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// Why the command failed.
    pub reason: String,
}

impl Payload for PluginFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
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

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_the_pump_have_their_wire_form() {
        let root = Root {
            session: uuid(SESSION),
        };
        let wire = || vec![text(SESSION)];
        assert_wire(&Reply::LoadRootSuccess(root), "/loadRoot.success", wire());
        assert_wire(
            &Reply::UnloadRootSuccess(root),
            "/unloadRoot.success",
            wire(),
        );
        let failure = RootFailure {
            session: uuid(SESSION),
            reason: "scsynth did not answer in time.".to_owned(),
        };
        let wire = || vec![text(SESSION), text("scsynth did not answer in time.")];
        assert_wire(
            &Reply::LoadRootFailure(failure.clone()),
            "/loadRoot.failure",
            wire(),
        );
        assert_wire(
            &Reply::UnloadRootFailure(failure),
            "/unloadRoot.failure",
            wire(),
        );
        let plugin = Plugin {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
        };
        let wire = || vec![text(SESSION), text(PLUGIN)];
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
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            reason: "The root group is not ready.".to_owned(),
        };
        let wire = || {
            vec![
                text(SESSION),
                text(PLUGIN),
                text("The root group is not ready."),
            ]
        };
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
            decode_error::<Reply>("/loadRoot.failure", vec![text(SESSION)]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "a string"
            }
        );
        let kind = decode_error::<Reply>("/loadPlugin.success", vec![text(SESSION), text("x")]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
    }
}
