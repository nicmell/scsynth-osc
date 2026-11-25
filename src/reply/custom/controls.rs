//! The answer of sc-app3's pump about the controls of the synth of an
//! element of a plugin: the values that scsynth holds.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_controls, read_id, read_uuid, write_uuid};
use crate::error::DecodeError;
use crate::reply::synth::RangeReading;

/// The arguments of `/control`: values of controls of the synth of an
/// element of a plugin, as scsynth holds them.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Controls {
    /// The plugin.
    pub plugin: Uuid,
    /// The id of the element of the synth. It cannot be empty.
    pub id: String,
    /// The ranges and their values, as the `/n_setn` that answers
    /// `/s_getn` carries them. The list cannot be empty.
    pub controls: Vec<RangeReading>,
}

impl Payload for Controls {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.string(&self.id);
        for reading in &self.controls {
            reading.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            id: read_id(reader)?,
            controls: read_controls(reader, RangeReading::read)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};
    use crate::types::Control;

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const ELEMENT: &str = "1bcdb1b42939e3";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answer_of_the_controls_has_its_wire_form() {
        let controls = Controls {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            controls: vec![
                RangeReading {
                    control: Control::from("freq"),
                    values: vec![330.0],
                },
                RangeReading {
                    control: Control::from("amps"),
                    values: vec![0.5, 0.25],
                },
            ],
        };
        assert_wire(
            &Reply::Control(controls),
            "/control",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("freq"),
                Int(1),
                Float(330.0),
                text("amps"),
                Int(2),
                Float(0.5),
                Float(0.25),
            ],
        );
    }

    #[test]
    fn an_id_and_the_controls_are_not_empty() {
        let kind = decode_error::<Reply>(
            "/control",
            vec![text(PLUGIN), text(""), text("freq"), Int(1), Float(330.0)],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Reply>("/control", vec![text(PLUGIN), text(ELEMENT)]),
            DecodeErrorKind::Missing {
                index: 2,
                expected: "a control index or name"
            }
        );
    }
}
