//! The commands of sc-app3's pump about the controls of the synth of an
//! element of a plugin: the values that a page sets, and the reads that it
//! asks for.

use uuid::Uuid;

use super::{read_id, read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::command::synth::RangeRequest;
use crate::error::DecodeError;
use crate::types::ControlSetting;

/// The arguments of `/setControl`: values of controls of the synth of an
/// element of a plugin, as a page sets them.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetControl {
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element of the synth. It cannot be empty.
    pub id: String,
    /// The values of controls, set in order, as `/n_set` sets them. The list
    /// cannot be empty.
    pub controls: Vec<ControlSetting>,
}

impl Payload for SetControl {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        for setting in &self.controls {
            setting.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            controls: read_controls(reader, ControlSetting::read)?,
        })
    }
}

/// The arguments of `/getControl`: ranges of controls of the synth of an
/// element of a plugin, which a page reads.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetControl {
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element of the synth. It cannot be empty.
    pub id: String,
    /// The ranges, as `/s_getn` names them. The list cannot be empty.
    pub controls: Vec<RangeRequest>,
}

impl Payload for GetControl {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        for range in &self.controls {
            range.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            controls: read_controls(reader, RangeRequest::read)?,
        })
    }
}

/// The controls of a message of the controls of the pump, each read by
/// `read` until none is left: one or more.
pub(crate) fn read_controls<T>(
    reader: &mut Reader<'_>,
    read: impl Fn(&mut Reader<'_>) -> Result<T, DecodeError>,
) -> Result<Vec<T>, DecodeError> {
    let mut controls = vec![read(reader)?];
    controls.extend(reader.rest(read)?);
    Ok(controls)
}

#[cfg(test)]
mod tests {
    use rosc::OscArray;
    use rosc::OscType::{Array, Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};
    use crate::types::{ArrayElement, Control, ControlValue};

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const ELEMENT: &str = "1bcdb1b42939e3";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_commands_of_the_controls_have_their_wire_form() {
        let set = SetControl {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            controls: vec![
                ControlSetting::new("freq", 330.0),
                ControlSetting::new(
                    "amps",
                    ControlValue::Array(vec![ArrayElement::Float(0.5), ArrayElement::Float(0.25)]),
                ),
            ],
        };
        assert_wire(
            &Command::SetControl(set),
            "/setControl",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("freq"),
                Float(330.0),
                text("amps"),
                Array(OscArray {
                    content: vec![Float(0.5), Float(0.25)],
                }),
            ],
        );
        let get = GetControl {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            controls: vec![
                RangeRequest {
                    control: Control::from("freq"),
                    count: 1,
                },
                RangeRequest {
                    control: Control::from("amps"),
                    count: 2,
                },
            ],
        };
        assert_wire(
            &Command::GetControl(get),
            "/getControl",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("freq"),
                Int(1),
                text("amps"),
                Int(2),
            ],
        );
    }

    #[test]
    fn an_id_and_the_controls_are_not_empty() {
        for (address, control) in [
            ("/setControl", vec![text("freq"), Float(330.0)]),
            ("/getControl", vec![text("freq"), Int(1)]),
        ] {
            let mut args = vec![text(PLUGIN), text("")];
            args.extend(control);
            let kind = decode_error::<Command>(address, args);
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
                "{address}: {kind:?}"
            );
            assert_eq!(
                decode_error::<Command>(address, vec![text(PLUGIN), text(ELEMENT)]),
                DecodeErrorKind::Missing {
                    index: 2,
                    expected: "a control index or name"
                },
                "{address}"
            );
        }
    }
}
