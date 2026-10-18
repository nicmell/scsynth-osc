//! OSC messages as an address and typed arguments, without the meaning
//! that [`Command`](crate::Command) and [`Reply`](crate::Reply) give them.

use rosc::{OscArray, OscMessage, OscType};

use crate::codec::{Message, sealed, tag};
use crate::error::{DecodeError, DecodeErrorKind};
use crate::packet::{decode_message, encode_message};

const EXPECTED: &str = "an int, a float, a double, a string, a blob or an array";

/// One OSC message: its address and its typed arguments. See
/// [Raw messages](crate#raw-messages).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawMessage {
    /// The address, such as `/sync`.
    pub address: String,
    /// The arguments, in order.
    pub args: Vec<RawArg>,
}

/// An argument of a [`RawMessage`], with its OSC type.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[non_exhaustive]
pub enum RawArg {
    /// A 32-bit integer, `i`.
    #[cfg_attr(feature = "serde", serde(rename = "i"))]
    Int(i32),
    /// A 32-bit float, `f`.
    #[cfg_attr(feature = "serde", serde(rename = "f"))]
    Float(f32),
    /// A 64-bit float, `d`.
    #[cfg_attr(feature = "serde", serde(rename = "d"))]
    Double(f64),
    /// A string, `s`.
    #[cfg_attr(feature = "serde", serde(rename = "s"))]
    String(String),
    /// A blob, `b`.
    #[cfg_attr(feature = "serde", serde(rename = "b"))]
    Blob(#[cfg_attr(feature = "serde", serde(with = "serde_bytes"))] Vec<u8>),
    /// An array of arguments, between `[` and `]`.
    #[cfg_attr(feature = "serde", serde(rename = "["))]
    Array(Vec<RawArg>),
}

impl RawMessage {
    /// The bytes of the message, as one OSC message.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        encode_message(osc_message(self))
    }

    /// The message in `bytes`, which must hold exactly one OSC message.
    ///
    /// # Errors
    ///
    /// When the bytes are not one OSC message, or an argument has another
    /// OSC type than those of [`RawArg`].
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        raw_message(&decode_message(bytes)?)
    }
}

impl Message for RawMessage {}

impl sealed::Codec for RawMessage {
    fn to_raw(&self) -> RawMessage {
        self.clone()
    }

    fn from_osc(message: &OscMessage) -> Result<Self, DecodeError> {
        raw_message(message)
    }

    fn to_osc(&self) -> OscMessage {
        osc_message(self)
    }

    fn from_raw(raw: &RawMessage) -> Result<Self, DecodeError> {
        Ok(raw.clone())
    }
}

/// `raw` as rosc writes it. Strings, the address too, end at their first
/// NUL byte.
pub(crate) fn osc_message(raw: &RawMessage) -> OscMessage {
    OscMessage {
        addr: up_to_nul(&raw.address).to_owned(),
        args: raw.args.iter().map(osc_arg).collect(),
    }
}

fn osc_arg(arg: &RawArg) -> OscType {
    match arg {
        RawArg::Int(value) => OscType::Int(*value),
        RawArg::Float(value) => OscType::Float(*value),
        RawArg::Double(value) => OscType::Double(*value),
        RawArg::String(value) => OscType::String(up_to_nul(value).to_owned()),
        RawArg::Blob(value) => OscType::Blob(value.clone()),
        RawArg::Array(values) => OscType::Array(OscArray {
            content: values.iter().map(osc_arg).collect(),
        }),
    }
}

fn up_to_nul(value: &str) -> &str {
    value.find('\0').map_or(value, |end| &value[..end])
}

/// The raw form of `message`, as rosc reads it.
fn raw_message(message: &OscMessage) -> Result<RawMessage, DecodeError> {
    let args = message
        .args
        .iter()
        .enumerate()
        .map(|(index, value)| {
            raw_arg(value).map_err(|found| {
                DecodeError::new(
                    &message.addr,
                    DecodeErrorKind::WrongType {
                        index,
                        expected: EXPECTED,
                        found,
                    },
                )
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(RawMessage {
        address: message.addr.clone(),
        args,
    })
}

/// `value` as a [`RawArg`], or the tag of the first value in it that has
/// no raw form.
fn raw_arg(value: &OscType) -> Result<RawArg, char> {
    Ok(match value {
        OscType::Int(value) => RawArg::Int(*value),
        OscType::Float(value) => RawArg::Float(*value),
        OscType::Double(value) => RawArg::Double(*value),
        OscType::String(value) => RawArg::String(value.clone()),
        OscType::Blob(value) => RawArg::Blob(value.clone()),
        OscType::Array(array) => RawArg::Array(
            array
                .content
                .iter()
                .map(raw_arg)
                .collect::<Result<_, _>>()?,
        ),
        other => return Err(tag(other)),
    })
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Bool, Int, Long};

    use super::*;
    use crate::command::buffer::BFree;
    use crate::command::synth::SNew;
    use crate::command::synth_def::DRecv;
    use crate::packet::{Bundle, Packet, Timetag};
    use crate::testing::bytes;
    use crate::{
        AddAction, ArrayElement, BufferId, BusMapping, Command, Completion, ControlSetting,
        ControlValue, NewNodeId, NodeId, Reply, SyncId,
    };

    fn text(value: &str) -> RawArg {
        RawArg::String(value.to_owned())
    }

    #[test]
    fn every_raw_argument_survives_the_bytes() {
        let raw = RawMessage {
            address: "/demo".to_owned(),
            args: vec![
                RawArg::Int(-1),
                RawArg::Float(0.5),
                RawArg::Double(0.25),
                text("x"),
                RawArg::Blob(vec![1, 2, 3]),
                RawArg::Array(vec![
                    RawArg::Int(1),
                    RawArg::Array(vec![text("c1")]),
                    RawArg::Array(vec![]),
                ]),
            ],
        };
        assert_eq!(RawMessage::decode(&raw.encode()), Ok(raw));
    }

    #[test]
    fn a_command_has_the_raw_form_of_its_bytes() {
        let new = Command::SNew(SNew {
            def: "default".to_owned(),
            id: NewNodeId::AUTO,
            action: AddAction::Head,
            target: NodeId(1),
            controls: vec![ControlSetting::new(
                "freqs",
                ControlValue::Array(vec![
                    ArrayElement::Float(440.0),
                    ArrayElement::Bus(BusMapping::Control(3)),
                ]),
            )],
        });
        let recv = Command::DRecv(DRecv {
            data: vec![1, 2, 3],
            completion: Some(Completion::Message(Box::new(Command::Status))),
        });
        let free = Command::BFree(BFree {
            buffer: BufferId::from(4),
            completion: Some(Completion::Bundle {
                time: Timetag::IMMEDIATELY,
                commands: vec![Command::Sync(SyncId(1))],
            }),
        });
        for command in [new, recv, free] {
            let raw = command.to_raw();
            assert_eq!(RawMessage::decode(&command.encode()).as_ref(), Ok(&raw));
            assert_eq!(raw.encode(), command.encode());
            assert_eq!(Command::from_raw(&raw), Ok(command));
        }
        assert_eq!(
            Command::Sync(SyncId(7)).to_raw(),
            RawMessage {
                address: "/sync".to_owned(),
                args: vec![RawArg::Int(7)],
            }
        );
    }

    #[test]
    fn from_raw_is_as_strict_as_decode() {
        let float = RawMessage {
            address: "/sync".to_owned(),
            args: vec![RawArg::Float(7.0)],
        };
        let error = Command::from_raw(&float).expect_err("a float is no int");
        assert_eq!(Command::decode(&float.encode()), Err(error.clone()));
        assert_eq!(
            error.kind(),
            &DecodeErrorKind::WrongType {
                index: 0,
                expected: "an int",
                found: 'f'
            }
        );
        let unknown = RawMessage {
            address: "/beat".to_owned(),
            args: vec![],
        };
        let error = Reply::from_raw(&unknown).expect_err("no such reply");
        assert_eq!(error.kind(), &DecodeErrorKind::UnknownAddress);
    }

    #[test]
    fn a_raw_message_takes_no_other_osc_type() {
        for (value, found) in [(Bool(true), 'T'), (Long(1), 'h')] {
            let error =
                RawMessage::decode(&bytes("/demo", vec![Int(1), value])).expect_err("another type");
            assert_eq!(
                error.kind(),
                &DecodeErrorKind::WrongType {
                    index: 1,
                    expected: EXPECTED,
                    found
                }
            );
        }
    }

    #[test]
    fn a_raw_string_ends_at_its_first_nul_byte() {
        let raw = RawMessage {
            address: "/demo\0x".to_owned(),
            args: vec![text("a\0b")],
        };
        let written = RawMessage {
            address: "/demo".to_owned(),
            args: vec![text("a")],
        };
        assert_eq!(RawMessage::decode(&raw.encode()), Ok(written));
    }

    #[test]
    fn a_packet_of_raw_messages_flattens_its_bundles() {
        let sync = |id| Packet::Message(Command::Sync(SyncId(id)));
        let packet = Packet::Bundle(Bundle {
            time: Timetag::IMMEDIATELY,
            packets: vec![
                sync(1),
                Packet::Bundle(Bundle {
                    time: Timetag::IMMEDIATELY,
                    packets: vec![sync(2)],
                }),
            ],
        });
        let raw = Packet::<RawMessage>::decode(&packet.encode()).expect("decodes");
        assert_eq!(
            raw.into_messages(),
            [1, 2].map(|id| Command::Sync(SyncId(id)).to_raw())
        );
    }
}
