//! What the tests of the modules share.

use std::fmt::Debug;

use rosc::{OscMessage, OscPacket, OscType};

use crate::codec::Message;
use crate::error::{DecodeError, DecodeErrorKind};
use crate::packet::Packet;

/// The bytes of one OSC message.
pub(crate) fn bytes(address: &str, args: Vec<OscType>) -> Vec<u8> {
    rosc::encoder::encode(&OscPacket::Message(OscMessage {
        addr: address.to_owned(),
        args,
    }))
    .expect("an OSC message encodes")
}

/// `message` is `address` with exactly `args` on the wire, and the bytes
/// decode back to `message`.
pub(crate) fn assert_wire<T: Message + PartialEq + Debug + Clone>(
    message: &T,
    address: &str,
    args: Vec<OscType>,
) {
    let expected = OscMessage {
        addr: address.to_owned(),
        args,
    };
    assert_eq!(message.to_osc(), expected, "{message:?}");
    assert_eq!(
        T::from_raw(&message.to_raw()).as_ref(),
        Ok(message),
        "{message:?}"
    );
    let packet = Packet::Message(message.clone());
    assert_eq!(
        Packet::<T>::decode(&packet.encode()),
        Ok(packet),
        "{message:?}"
    );
}

/// The kind of error that decoding `address` with `args` as a `T` gives.
pub(crate) fn decode_error<T: Message + Debug>(
    address: &str,
    args: Vec<OscType>,
) -> DecodeErrorKind {
    let message = OscMessage {
        addr: address.to_owned(),
        args,
    };
    let error: DecodeError = T::from_osc(&message).expect_err("the message does not decode");
    assert_eq!(error.address(), address);
    error.kind().clone()
}

/// `address` with `args`, decoded as a `T`.
pub(crate) fn decode<T: Message + Debug>(address: &str, args: Vec<OscType>) -> T {
    let message = OscMessage {
        addr: address.to_owned(),
        args,
    };
    T::from_osc(&message).expect("the message decodes")
}
