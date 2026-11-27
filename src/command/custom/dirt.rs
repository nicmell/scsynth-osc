//! The commands of sc-app3's pump about the Dirt of an element of a
//! plugin: the Dirt that the pump starts in the group of the element, by
//! the node of that group, on a block of node ids that the page gives, and
//! the sounds that it plays on it.

use rosc::OscType;
use uuid::Uuid;

use super::{read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::raw::RawArg;

/// The arguments of `/loadDirt`: the Dirt of an element of a plugin, which
/// plays in the group of the element.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadDirt {
    /// The plugin.
    pub plugin: Uuid,
    /// The node id of the group of the element.
    pub node: i32,
    /// The block of node ids of the Dirt.
    pub block: i32,
    /// The number of orbits of the Dirt.
    pub orbits: i32,
    /// The first of the output buses of the Dirt.
    pub out: i32,
}

impl Payload for LoadDirt {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.int(self.node);
        writer.int(self.block);
        writer.int(self.orbits);
        writer.int(self.out);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            node: reader.int()?,
            block: reader.int()?,
            orbits: reader.int()?,
            out: reader.int()?,
        })
    }
}

/// The arguments of `/dirt.play`: a sound for the Dirt of an element of a
/// plugin, at a cycle of the tempo of the session.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirtPlay {
    /// The plugin.
    pub plugin: Uuid,
    /// The node id of the group of the element.
    pub node: i32,
    /// The cycle at which the sound starts.
    pub cycle: f64,
    /// The number of sounds of the element before this one that start at
    /// the same cycle: 0 or more.
    pub index: i32,
    /// The length of the sound, in cycles.
    pub span: f64,
    /// The parameters of the sound, in order.
    pub parameters: Vec<DirtParameter>,
}

impl Payload for DirtPlay {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.plugin);
        writer.int(self.node);
        writer.double(self.cycle);
        writer.int(self.index);
        writer.double(self.span);
        for parameter in &self.parameters {
            writer.string(&parameter.name);
            writer.push(parameter.value.to_raw());
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            plugin: read_uuid(reader)?,
            node: reader.int()?,
            cycle: reader.double()?,
            index: read_index(reader)?,
            span: reader.double()?,
            parameters: reader.rest(DirtParameter::read)?,
        })
    }
}

/// A parameter of a sound of a Dirt: its name and its value, such as `s`
/// and `"bd"`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirtParameter {
    /// The name. It cannot be empty.
    pub name: String,
    /// The value.
    pub value: DirtValue,
}

impl DirtParameter {
    /// The parameter `name` set to `value`.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<DirtValue>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let index = reader.index();
        let name = reader.string()?;
        if name.is_empty() {
            return Err(reader.invalid(index, "an empty name of a parameter"));
        }
        Ok(Self {
            name,
            value: DirtValue::read(reader)?,
        })
    }
}

/// The value of a [`DirtParameter`], with its OSC type.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum DirtValue {
    /// A 32-bit integer, `i`.
    Int(i32),
    /// A 32-bit float, `f`.
    Float(f32),
    /// A 64-bit float, `d`.
    Double(f64),
    /// A string, `s`.
    String(String),
}

impl DirtValue {
    /// The value as the raw argument that carries it.
    #[must_use]
    pub fn to_raw(&self) -> RawArg {
        match self {
            Self::Int(value) => RawArg::Int(*value),
            Self::Float(value) => RawArg::Float(*value),
            Self::Double(value) => RawArg::Double(*value),
            Self::String(value) => RawArg::String(value.clone()),
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        const EXPECTED: &str = "an int, a float, a double or a string";
        match reader.any(EXPECTED)? {
            OscType::Int(value) => Ok(Self::Int(*value)),
            OscType::Float(value) => Ok(Self::Float(*value)),
            OscType::Double(value) => Ok(Self::Double(*value)),
            OscType::String(value) => Ok(Self::String(value.clone())),
            other => Err(reader.wrong(other, EXPECTED)),
        }
    }
}

impl From<i32> for DirtValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<f32> for DirtValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for DirtValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<&str> for DirtValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for DirtValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// The index of a sound among those that start at its cycle: 0 or more.
fn read_index(reader: &mut Reader<'_>) -> Result<i32, DecodeError> {
    let at = reader.index();
    let index = reader.int()?;
    if index < 0 {
        return Err(reader.invalid(
            at,
            format!("{index} is not the index of a sound: 0 or more"),
        ));
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Array, Blob, Bool, Double, Float, Int, Long, String as Text, Time};
    use rosc::{OscArray, OscTime};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const NODE: i32 = 67_240_961;

    const VALUE: &str = "an int, a float, a double or a string";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> OscType {
        Text(value.to_owned())
    }

    /// The arguments of a sound of the group [`NODE`] at cycle 4 with
    /// `index`, then `tail`.
    fn sound(index: i32, tail: Vec<OscType>) -> Vec<OscType> {
        let mut args = vec![
            text(PLUGIN),
            Int(NODE),
            Double(4.0),
            Int(index),
            Double(0.5),
        ];
        args.extend(tail);
        args
    }

    #[test]
    fn the_commands_of_the_dirt_have_their_wire_form() {
        let load = LoadDirt {
            plugin: uuid(PLUGIN),
            node: NODE,
            block: 1,
            orbits: 2,
            out: 0,
        };
        assert_wire(
            &Command::LoadDirt(load),
            "/loadDirt",
            vec![text(PLUGIN), Int(NODE), Int(1), Int(2), Int(0)],
        );
        let play = DirtPlay {
            plugin: uuid(PLUGIN),
            node: NODE,
            cycle: 12.25,
            index: 1,
            span: 0.25,
            parameters: vec![
                DirtParameter::new("s", "bd"),
                DirtParameter::new("n", 3),
                DirtParameter::new("speed", 1.5_f32),
                DirtParameter::new("gain", 0.75),
            ],
        };
        let head = vec![text(PLUGIN), Int(NODE), Double(12.25), Int(1), Double(0.25)];
        let mut wire = head.clone();
        wire.extend([
            text("s"),
            text("bd"),
            text("n"),
            Int(3),
            text("speed"),
            Float(1.5),
            text("gain"),
            Double(0.75),
        ]);
        assert_wire(&Command::DirtPlay(play.clone()), "/dirt.play", wire);
        let bare = DirtPlay {
            parameters: Vec::new(),
            ..play
        };
        assert_wire(&Command::DirtPlay(bare), "/dirt.play", head);
    }

    #[test]
    fn a_node_is_an_int() {
        for (address, args) in [
            (
                "/loadDirt",
                vec![text(PLUGIN), text("a"), Int(0), Int(1), Int(0)],
            ),
            (
                "/dirt.play",
                vec![text(PLUGIN), text("a"), Double(4.0), Int(0), Double(0.5)],
            ),
        ] {
            assert_eq!(
                decode_error::<Command>(address, args),
                DecodeErrorKind::WrongType {
                    index: 1,
                    expected: "an int",
                    found: 's'
                },
                "{address}"
            );
        }
    }

    #[test]
    fn a_sound_takes_doubles_and_an_index_of_0_or_more() {
        let kind = decode_error::<Command>("/dirt.play", sound(-1, vec![]));
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 3, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Command>(
                "/dirt.play",
                vec![text(PLUGIN), Int(NODE), Float(4.0), Int(0), Double(0.5)]
            ),
            DecodeErrorKind::WrongType {
                index: 2,
                expected: "a double",
                found: 'f'
            }
        );
    }

    #[test]
    fn the_parameters_of_a_sound_are_pairs_of_a_name_and_a_value() {
        assert_eq!(
            decode_error::<Command>(
                "/dirt.play",
                sound(0, vec![text("s"), text("bd"), text("n")])
            ),
            DecodeErrorKind::Missing {
                index: 8,
                expected: VALUE
            }
        );
        assert_eq!(
            decode_error::<Command>("/dirt.play", sound(0, vec![Int(1), text("bd")])),
            DecodeErrorKind::WrongType {
                index: 5,
                expected: "a string",
                found: 'i'
            }
        );
        let kind = decode_error::<Command>("/dirt.play", sound(0, vec![text(""), text("bd")]));
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 5, .. }),
            "{kind:?}"
        );
        let time = OscTime {
            seconds: 1,
            fractional: 0,
        };
        let array = OscArray {
            content: vec![Int(1)],
        };
        for (value, found) in [
            (Blob(vec![1]), 'b'),
            (Time(time), 't'),
            (Long(1), 'h'),
            (Bool(true), 'T'),
            (Array(array), '['),
        ] {
            let tail = vec![text("s"), text("bd"), text("n"), value];
            assert_eq!(
                decode_error::<Command>("/dirt.play", sound(0, tail)),
                DecodeErrorKind::WrongType {
                    index: 8,
                    expected: VALUE,
                    found
                }
            );
        }
    }
}
