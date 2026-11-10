//! The commands of sc-app3's pump about the nodes of the elements of a
//! plugin: a group or a synth that the pump makes in scsynth, in the group
//! of the plugin or in the group of a parent element.

use uuid::Uuid;

use super::{read_name, read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::ControlSetting;

/// The arguments of `/loadGroup`: the group of an element of a plugin.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadGroup {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element. It cannot be empty.
    pub id: String,
    /// The id of the element of the group that holds it, or empty for the
    /// group of the plugin.
    pub parent: String,
}

impl Payload for LoadGroup {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        writer.string(&self.parent);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            parent: reader.string()?,
        })
    }
}

/// The arguments of `/loadSynth`: the synth of an element of a plugin, of
/// a synth definition, with the initial values of its controls.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadSynth {
    /// The session.
    pub session: Uuid,
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element. It cannot be empty.
    pub id: String,
    /// The id of the element of the group that holds it, or empty for the
    /// group of the plugin.
    pub parent: String,
    /// The name of the synth definition: 1 to 255 bytes.
    pub def: String,
    /// The initial values of controls, set in order, as `/s_new` sets them.
    pub controls: Vec<ControlSetting>,
}

impl Payload for LoadSynth {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        writer.string(&self.parent);
        writer.string(&self.def);
        for setting in &self.controls {
            setting.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            parent: reader.string()?,
            def: read_name(reader)?,
            controls: reader.rest(ControlSetting::read)?,
        })
    }
}

/// The id of an element, which cannot be empty.
pub(crate) fn read_id(reader: &mut Reader<'_>) -> Result<String, DecodeError> {
    let index = reader.index();
    let id = reader.string()?;
    if id.is_empty() {
        return Err(reader.invalid(index, "an empty id of an element"));
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode, decode_error};

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_commands_of_the_nodes_have_their_wire_form() {
        let group = LoadGroup {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            id: "39fc2565fa0c9".to_owned(),
            parent: String::new(),
        };
        assert_wire(
            &Command::LoadGroup(group),
            "/loadGroup",
            vec![text(SESSION), text(PLUGIN), text("39fc2565fa0c9"), text("")],
        );
        let synth = LoadSynth {
            session: uuid(SESSION),
            plugin: uuid(PLUGIN),
            id: "1bcdb1b42939e3".to_owned(),
            parent: "39fc2565fa0c9".to_owned(),
            def: "32183c940ae4".to_owned(),
            controls: vec![
                ControlSetting::new("freq", 261.0),
                ControlSetting::new(3, 1),
            ],
        };
        assert_wire(
            &Command::LoadSynth(synth),
            "/loadSynth",
            vec![
                text(SESSION),
                text(PLUGIN),
                text("1bcdb1b42939e3"),
                text("39fc2565fa0c9"),
                text("32183c940ae4"),
                text("freq"),
                Float(261.0),
                Int(3),
                Int(1),
            ],
        );
    }

    #[test]
    fn an_id_is_not_empty_and_a_definition_takes_1_to_255_bytes() {
        let group = |id: &str| vec![text(SESSION), text(PLUGIN), text(id), text("")];
        let kind = decode_error::<Command>("/loadGroup", group(""));
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode::<Command>("/loadGroup", group("a")),
            Command::LoadGroup(LoadGroup {
                session: uuid(SESSION),
                plugin: uuid(PLUGIN),
                id: "a".to_owned(),
                parent: String::new(),
            })
        );
        let synth = |def: &str| vec![text(SESSION), text(PLUGIN), text("a"), text(""), text(def)];
        for def in [String::new(), "n".repeat(256)] {
            let kind = decode_error::<Command>("/loadSynth", synth(&def));
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 4, .. }),
                "{}: {kind:?}",
                def.len()
            );
        }
        let kind = decode_error::<Command>(
            "/loadSynth",
            vec![
                text("not a uuid"),
                text(PLUGIN),
                text("a"),
                text(""),
                text("b"),
            ],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
            "{kind:?}"
        );
    }
}
