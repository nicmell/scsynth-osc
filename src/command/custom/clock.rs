//! The command of sc-app3's pump about the clock of a session: the ping of
//! a page, which the pump answers with `/clock.pong`.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/clock.ping`: the time of the page at the send.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClockPing {
    /// The time of the page when it sent the ping, in milliseconds since the
    /// Unix epoch.
    pub sent_ms: f64,
}

impl Payload for ClockPing {
    fn write(&self, writer: &mut Writer) {
        writer.double(self.sent_ms);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            sent_ms: reader.double()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Double, Float};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn a_ping_carries_the_time_of_the_page_as_a_double() {
        let ping = ClockPing {
            sent_ms: 1_792_605_767_123.25,
        };
        assert_wire(
            &Command::ClockPing(ping),
            "/clock.ping",
            vec![Double(1_792_605_767_123.25)],
        );
        assert_eq!(
            decode_error::<Command>("/clock.ping", vec![Float(1.0)]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "a double",
                found: 'f'
            }
        );
    }
}
