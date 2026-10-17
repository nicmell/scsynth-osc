//! The replies of the synth commands: `/s_get` and `/s_getn` answer with
//! the `/n_set` and `/n_setn` that would set the values they read.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{Control, NodeId};

/// The arguments of the `/n_set` that answers `/s_get`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NSet {
    /// The synth, by its ID, also when the query named it -1.
    pub node: NodeId,
    /// The controls, as the query named them, and their values.
    pub controls: Vec<ValueReading>,
}

/// A control of `/s_get`, and its value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValueReading {
    /// The control.
    pub control: Control,
    /// Its value: 0 for a control that the synth does not have.
    pub value: f32,
}

impl Payload for NSet {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for reading in &self.controls {
            reading.control.write(writer);
            writer.float(reading.value);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let controls = reader.rest(|reader| {
            Ok(ValueReading {
                control: Control::read(reader)?,
                value: reader.float()?,
            })
        })?;
        Ok(Self { node, controls })
    }
}

/// The arguments of the `/n_setn` that answers `/s_getn`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NSetn {
    /// The synth, by its ID, also when the query named it -1.
    pub node: NodeId,
    /// The ranges, as the query named them, and their values.
    pub ranges: Vec<RangeReading>,
}

/// A range of `/s_getn`, and its values.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeReading {
    /// The first control of the range.
    pub control: Control,
    /// The values: 0 for a control that the synth does not have.
    pub values: Vec<f32>,
}

impl Payload for NSetn {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for reading in &self.ranges {
            reading.control.write(writer);
            writer.count(reading.values.len());
            for value in &reading.values {
                writer.float(*value);
            }
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let ranges = reader.rest(|reader| {
            let control = Control::read(reader)?;
            let count = reader.count()?;
            let values = (0..count)
                .map(|_| reader.float())
                .collect::<Result<_, _>>()?;
            Ok(RangeReading { control, values })
        })?;
        Ok(Self { node, ranges })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_carry_floats() {
        let set = NSet {
            node: NodeId(1000),
            controls: vec![
                ValueReading {
                    control: Control::from("freq"),
                    value: 440.0,
                },
                ValueReading {
                    control: Control::Index(1),
                    value: 0.0,
                },
            ],
        };
        let wire = vec![Int(1000), text("freq"), Float(440.0), Int(1), Float(0.0)];
        assert_wire(&Reply::NSet(set), "/n_set", wire);
        let setn = NSetn {
            node: NodeId(1000),
            ranges: vec![RangeReading {
                control: Control::from("freqs"),
                values: vec![1.0, 2.0],
            }],
        };
        let wire = vec![Int(1000), text("freqs"), Int(2), Float(1.0), Float(2.0)];
        assert_wire(&Reply::NSetn(setn), "/n_setn", wire);
    }

    #[test]
    fn an_answer_with_an_int_value_or_a_negative_count_is_refused() {
        assert_eq!(
            decode_error::<Reply>("/n_set", vec![Int(1000), text("freq"), Int(440)]),
            DecodeErrorKind::WrongType {
                index: 2,
                expected: "a float",
                found: 'i'
            }
        );
        let kind = decode_error::<Reply>("/n_setn", vec![Int(1000), text("freqs"), Int(-1)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
    }
}
