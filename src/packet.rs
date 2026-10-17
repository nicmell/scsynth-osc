//! Packets: one message, or a bundle of packets with a time tag.

use rosc::{OscBundle, OscMessage, OscPacket, OscTime};

use crate::codec::Message;
use crate::error::{DecodeError, DecodeErrorKind};

/// An OSC time tag: seconds since 1 January 1900, and the fraction of the
/// next second in units of 2⁻³² seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timetag {
    /// Seconds since 1 January 1900.
    pub seconds: u32,
    /// The fraction of the next second, in units of 2⁻³² seconds.
    pub fraction: u32,
}

impl Timetag {
    /// The time tag that OSC reads as "at once": 0 seconds and a fraction
    /// of 1.
    pub const IMMEDIATELY: Self = Self {
        seconds: 0,
        fraction: 1,
    };

    /// Whether scsynth runs a bundle with this time tag at once instead of
    /// scheduling it. It does so for the time tags 0 and 1
    /// (`SC_CoreAudio.cpp`, in `PerformOSCBundle`).
    pub fn is_immediate(self) -> bool {
        self.seconds == 0 && self.fraction <= 1
    }
}

impl From<OscTime> for Timetag {
    fn from(time: OscTime) -> Self {
        Self {
            seconds: time.seconds,
            fraction: time.fractional,
        }
    }
}

impl From<Timetag> for OscTime {
    fn from(time: Timetag) -> Self {
        Self {
            seconds: time.seconds,
            fractional: time.fraction,
        }
    }
}

/// One message, or a bundle of packets. `T` is [`crate::Command`] or
/// [`crate::Reply`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Packet<T> {
    /// One message.
    Message(T),
    /// A bundle of packets.
    Bundle(Bundle<T>),
}

/// A bundle: packets that the server runs together, at its time tag.
/// scsynth runs the packets of a bundle that it receives from the network,
/// and of every bundle nested in it, in order.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bundle<T> {
    /// When the packets run.
    pub time: Timetag,
    /// The packets, in the order they run.
    pub packets: Vec<Packet<T>>,
}

impl<T> Packet<T> {
    /// The messages of the packet, in order, its bundles flattened.
    pub fn into_messages(self) -> Vec<T> {
        let mut messages = Vec::new();
        let mut pending = vec![self];
        while let Some(packet) = pending.pop() {
            match packet {
                Self::Message(message) => messages.push(message),
                Self::Bundle(bundle) => pending.extend(bundle.packets.into_iter().rev()),
            }
        }
        messages
    }
}

impl<T: Message> Packet<T> {
    /// The bytes of the packet.
    ///
    /// # Panics
    ///
    /// On a list of more than `i32::MAX` values, which no OSC packet holds.
    pub fn encode(&self) -> Vec<u8> {
        encode_osc(&self.to_osc())
    }

    /// The packet in `bytes`, which must hold exactly one OSC packet.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::from_osc(&decode_osc(bytes)?)
    }

    fn to_osc(&self) -> OscPacket {
        match self {
            Self::Message(message) => OscPacket::Message(message.to_osc()),
            Self::Bundle(bundle) => OscPacket::Bundle(OscBundle {
                timetag: bundle.time.into(),
                content: bundle.packets.iter().map(Self::to_osc).collect(),
            }),
        }
    }

    fn from_osc(packet: &OscPacket) -> Result<Self, DecodeError> {
        match packet {
            OscPacket::Message(message) => T::from_osc(message).map(Self::Message),
            OscPacket::Bundle(bundle) => Ok(Self::Bundle(Bundle {
                time: bundle.timetag.into(),
                packets: bundle
                    .content
                    .iter()
                    .map(Self::from_osc)
                    .collect::<Result<_, _>>()?,
            })),
        }
    }
}

pub(crate) fn encode_osc(packet: &OscPacket) -> Vec<u8> {
    rosc::encoder::encode(packet).expect("rosc encodes into a Vec without failing")
}

pub(crate) fn encode_message(message: OscMessage) -> Vec<u8> {
    encode_osc(&OscPacket::Message(message))
}

/// The one OSC packet in `bytes`, with nothing after it.
pub(crate) fn decode_osc(bytes: &[u8]) -> Result<OscPacket, DecodeError> {
    let osc = |reason: String| DecodeError::new("", DecodeErrorKind::Osc(reason));
    let (rest, packet) =
        rosc::decoder::decode_udp(bytes).map_err(|error| osc(error.to_string()))?;
    if !rest.is_empty() {
        return Err(osc(format!("{} bytes after the packet", rest.len())));
    }
    Ok(packet)
}

/// The one OSC message in `bytes`.
pub(crate) fn decode_message(bytes: &[u8]) -> Result<OscMessage, DecodeError> {
    match decode_osc(bytes)? {
        OscPacket::Message(message) => Ok(message),
        OscPacket::Bundle(_) => Err(DecodeError::new("", DecodeErrorKind::Bundle)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use crate::command::top_level::SyncId;

    #[test]
    fn the_messages_of_a_packet_flatten_in_wire_order() {
        let sync = |id| Packet::Message(Command::Sync(SyncId(id)));
        let inner = Packet::Bundle(Bundle {
            time: Timetag::IMMEDIATELY,
            packets: vec![sync(2), sync(3)],
        });
        let outer = Packet::Bundle(Bundle {
            time: Timetag {
                seconds: 7,
                fraction: 9,
            },
            packets: vec![sync(1), inner, sync(4)],
        });
        let decoded = Packet::<Command>::decode(&outer.encode()).expect("decodes");
        assert_eq!(
            decoded.into_messages(),
            (1..=4)
                .map(|id| Command::Sync(SyncId(id)))
                .collect::<Vec<_>>()
        );
        assert_eq!(sync(5).into_messages(), [Command::Sync(SyncId(5))]);
    }
}
