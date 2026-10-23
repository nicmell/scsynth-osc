//! The message of the clock of an sc-app3 session: the pong of the pump to
//! the ping of a page.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/clock.pong`: the answer of the pump to a
/// `/clock.ping`, with server time and the timing of the session.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClockPong {
    /// The `sent_ms` of the ping, as it came.
    pub sent_ms: f64,
    /// Server time when the pump answered: the clock of the bridge, in
    /// milliseconds since the Unix epoch.
    pub server_ms: f64,
    /// The beat of the pump, in milliseconds: the page pings once per beat.
    pub beat_ms: i32,
    /// The latency of the session, in milliseconds: a page schedules each
    /// bundle at server time now plus the latency.
    pub latency_ms: i32,
}

impl Payload for ClockPong {
    fn write(&self, writer: &mut Writer) {
        writer.double(self.sent_ms);
        writer.double(self.server_ms);
        writer.int(self.beat_ms);
        writer.int(self.latency_ms);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            sent_ms: reader.double()?,
            server_ms: reader.double()?,
            beat_ms: reader.int()?,
            latency_ms: reader.int()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Double, Int};

    use super::*;
    use crate::Reply;
    use crate::testing::assert_wire;

    #[test]
    fn a_pong_carries_server_time_and_the_timing_of_the_session() {
        let pong = ClockPong {
            sent_ms: 1_792_605_767_123.25,
            server_ms: 1_792_605_767_124.5,
            beat_ms: 1000,
            latency_ms: 200,
        };
        let wire = vec![
            Double(1_792_605_767_123.25),
            Double(1_792_605_767_124.5),
            Int(1000),
            Int(200),
        ];
        assert_wire(&Reply::ClockPong(pong), "/clock.pong", wire);
    }
}
