//! OSC messages as an address and typed arguments, without the meaning
//! that [`Command`](crate::Command) and [`Reply`](crate::Reply) give them.

use rosc::{OscArray, OscColor, OscMessage, OscMidiMessage, OscType};

use crate::codec::{Message, sealed};
use crate::error::DecodeError;
use crate::packet::{Timetag, decode_message, encode_message};

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
    /// A 64-bit integer, `h`.
    #[cfg_attr(feature = "serde", serde(rename = "h"))]
    Long(i64),
    /// A time tag, `t`.
    #[cfg_attr(feature = "serde", serde(rename = "t"))]
    Time(Timetag),
    /// A character, `c`.
    #[cfg_attr(feature = "serde", serde(rename = "c"))]
    Char(char),
    /// A color, `r`.
    #[cfg_attr(feature = "serde", serde(rename = "r"))]
    Color(Color),
    /// A MIDI message, `m`.
    #[cfg_attr(feature = "serde", serde(rename = "m"))]
    Midi(MidiMessage),
    /// True, `T`, which has no data.
    #[cfg_attr(feature = "serde", serde(rename = "T"))]
    True,
    /// False, `F`, which has no data.
    #[cfg_attr(feature = "serde", serde(rename = "F"))]
    False,
    /// Nil, `N`, which has no data.
    #[cfg_attr(feature = "serde", serde(rename = "N"))]
    Nil,
    /// Infinitum, `I`, which has no data.
    #[cfg_attr(feature = "serde", serde(rename = "I"))]
    Inf,
}

/// A color, the argument `r`: one byte per channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// The red channel.
    pub red: u8,
    /// The green channel.
    pub green: u8,
    /// The blue channel.
    pub blue: u8,
    /// The alpha channel.
    pub alpha: u8,
}

/// A MIDI message, the argument `m`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MidiMessage {
    /// The port id.
    pub port: u8,
    /// The status byte.
    pub status: u8,
    /// The first data byte.
    pub data1: u8,
    /// The second data byte.
    pub data2: u8,
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
    /// When the bytes are not one OSC message.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(raw_message(&decode_message(bytes)?))
    }
}

impl Message for RawMessage {}

impl sealed::Codec for RawMessage {
    fn to_raw(&self) -> RawMessage {
        self.clone()
    }

    fn from_osc(message: &OscMessage) -> Result<Self, DecodeError> {
        Ok(raw_message(message))
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
        RawArg::Long(value) => OscType::Long(*value),
        RawArg::Time(time) => OscType::Time(time.to_osc()),
        RawArg::Char(value) => OscType::Char(*value),
        RawArg::Color(color) => OscType::Color(OscColor {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: color.alpha,
        }),
        RawArg::Midi(midi) => OscType::Midi(OscMidiMessage {
            port: midi.port,
            status: midi.status,
            data1: midi.data1,
            data2: midi.data2,
        }),
        RawArg::True => OscType::Bool(true),
        RawArg::False => OscType::Bool(false),
        RawArg::Nil => OscType::Nil,
        RawArg::Inf => OscType::Inf,
    }
}

fn up_to_nul(value: &str) -> &str {
    value.find('\0').map_or(value, |end| &value[..end])
}

/// The raw form of `message`, as rosc reads it.
fn raw_message(message: &OscMessage) -> RawMessage {
    RawMessage {
        address: message.addr.clone(),
        args: message.args.iter().map(raw_arg).collect(),
    }
}

fn raw_arg(value: &OscType) -> RawArg {
    match value {
        OscType::Int(value) => RawArg::Int(*value),
        OscType::Float(value) => RawArg::Float(*value),
        OscType::String(value) => RawArg::String(value.clone()),
        OscType::Blob(value) => RawArg::Blob(value.clone()),
        OscType::Time(time) => RawArg::Time(Timetag::from_osc(*time)),
        OscType::Long(value) => RawArg::Long(*value),
        OscType::Double(value) => RawArg::Double(*value),
        OscType::Char(value) => RawArg::Char(*value),
        OscType::Color(color) => RawArg::Color(Color {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: color.alpha,
        }),
        OscType::Midi(midi) => RawArg::Midi(MidiMessage {
            port: midi.port,
            status: midi.status,
            data1: midi.data1,
            data2: midi.data2,
        }),
        OscType::Bool(true) => RawArg::True,
        OscType::Bool(false) => RawArg::False,
        OscType::Array(array) => RawArg::Array(array.content.iter().map(raw_arg).collect()),
        OscType::Nil => RawArg::Nil,
        OscType::Inf => RawArg::Inf,
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscTime;

    use super::*;
    use crate::command::buffer::BFree;
    use crate::command::synth::SNew;
    use crate::command::synth_def::DRecv;
    use crate::error::DecodeErrorKind;
    use crate::packet::{Bundle, Packet};
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
                    RawArg::Array(vec![text("c1"), RawArg::True]),
                    RawArg::Array(vec![]),
                ]),
                RawArg::Long(-2),
                RawArg::Time(Timetag {
                    seconds: 1,
                    fraction: 2,
                }),
                RawArg::Char('x'),
                RawArg::Color(Color {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 4,
                }),
                RawArg::Midi(MidiMessage {
                    port: 1,
                    status: 0x90,
                    data1: 60,
                    data2: 100,
                }),
                RawArg::True,
                RawArg::False,
                RawArg::Nil,
                RawArg::Inf,
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
        for (arg, found) in [(RawArg::Float(7.0), 'f'), (RawArg::Long(7), 'h')] {
            let sync = RawMessage {
                address: "/sync".to_owned(),
                args: vec![arg],
            };
            let error = Command::from_raw(&sync).expect_err("no int");
            assert_eq!(Command::decode(&sync.encode()), Err(error.clone()));
            assert_eq!(
                error.kind(),
                &DecodeErrorKind::WrongType {
                    index: 0,
                    expected: "an int",
                    found
                }
            );
        }
        let unknown = RawMessage {
            address: "/beat".to_owned(),
            args: vec![],
        };
        let error = Reply::from_raw(&unknown).expect_err("no such reply");
        assert_eq!(error.kind(), &DecodeErrorKind::UnknownAddress);
    }

    #[test]
    fn every_osc_type_has_a_raw_form_that_writes_the_same_bytes() {
        let written = bytes(
            "/demo",
            vec![
                OscType::Int(1),
                OscType::Float(0.5),
                OscType::String("s".to_owned()),
                OscType::Blob(vec![1]),
                OscType::Time(OscTime {
                    seconds: 1,
                    fractional: 2,
                }),
                OscType::Long(-3),
                OscType::Double(0.25),
                OscType::Char('x'),
                OscType::Color(OscColor {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 4,
                }),
                OscType::Midi(OscMidiMessage {
                    port: 1,
                    status: 0x90,
                    data1: 60,
                    data2: 100,
                }),
                OscType::Bool(true),
                OscType::Bool(false),
                OscType::Array(OscArray {
                    content: vec![OscType::Int(1), OscType::Nil],
                }),
                OscType::Nil,
                OscType::Inf,
            ],
        );
        let raw = RawMessage::decode(&written).expect("every OSC type reads");
        assert_eq!(raw.encode(), written);
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
