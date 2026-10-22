//! The messages of the clock of an sc-app3 session: the pong of the pump,
//! and the reply of the clock synth of the session, which scsynth sends
//! with `SendReply` to every registered client.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::NodeId;

/// The arguments of `/clock.pong`: the answer of the pump to a
/// `/clock.ping`, with the clock of the session.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClockPong {
    /// The `sent_ms` of the ping, as it came.
    pub sent_ms: f64,
    /// Server time when the pump answered: the clock of the bridge, in
    /// milliseconds since the Unix epoch.
    pub server_ms: f64,
    /// The sample of the timeline heard at `server_ms`.
    pub sample: f64,
    /// The samples of the timeline per second of server time. 0 while the
    /// pump knows no map.
    pub rate: f64,
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
        writer.double(self.sample);
        writer.double(self.rate);
        writer.int(self.beat_ms);
        writer.int(self.latency_ms);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            sent_ms: reader.double()?,
            server_ms: reader.double()?,
            sample: reader.double()?,
            rate: reader.double()?,
            beat_ms: reader.int()?,
            latency_ms: reader.int()?,
        })
    }
}

/// The arguments of `/tick`, which the clock synth sends every few blocks,
/// with the last click on its probe bus.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tick {
    /// The clock synth.
    pub node: NodeId,
    /// The reply id of its `SendReply`: −1.
    pub reply: i32,
    /// The count of ticks since the clock started, from 1.
    pub count: f32,
    /// The first sample of the block of the tick, modulo 2^24.
    pub phase: f32,
    /// The sample of the last click on the probe bus, modulo 2^24, or 0
    /// before the first click.
    pub click_phase: f32,
    /// The height of the last click on the probe bus, which names it, or 0
    /// before the first click.
    pub click_id: f32,
}

impl Payload for Tick {
    fn write(&self, writer: &mut Writer) {
        let values = [self.count, self.phase, self.click_phase, self.click_id];
        write_reply(writer, self.node, self.reply, values);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let (node, reply, [count, phase, click_phase, click_id]) = read_reply(reader)?;
        Ok(Self {
            node,
            reply,
            count,
            phase,
            click_phase,
            click_id,
        })
    }
}

/// The arguments of a reply of `SendReply`: the node, the reply id, then
/// the values.
fn write_reply<const N: usize>(writer: &mut Writer, node: NodeId, reply: i32, values: [f32; N]) {
    node.write(writer);
    writer.int(reply);
    for value in values {
        writer.float(value);
    }
}

fn read_reply<const N: usize>(
    reader: &mut Reader<'_>,
) -> Result<(NodeId, i32, [f32; N]), DecodeError> {
    let node = NodeId::read(reader)?;
    let reply = reader.int()?;
    let mut values = [0.0; N];
    for value in &mut values {
        *value = reader.float()?;
    }
    Ok((node, reply, values))
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Double, Float, Int};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn a_pong_carries_the_clock_of_the_session() {
        let pong = ClockPong {
            sent_ms: 1_792_605_767_123.25,
            server_ms: 1_792_605_767_124.5,
            sample: 5_000_000_000.0,
            rate: 48_000.5,
            beat_ms: 1000,
            latency_ms: 200,
        };
        let wire = vec![
            Double(1_792_605_767_123.25),
            Double(1_792_605_767_124.5),
            Double(5_000_000_000.0),
            Double(48_000.5),
            Int(1000),
            Int(200),
        ];
        assert_wire(&Reply::ClockPong(pong), "/clock.pong", wire);
    }

    #[test]
    fn a_tick_has_its_wire_form() {
        let tick = Tick {
            node: NodeId(67_174_401),
            reply: -1,
            count: 3.0,
            phase: 2048.0,
            click_phase: 1733.0,
            click_id: 9.0,
        };
        let wire = vec![
            Int(67_174_401),
            Int(-1),
            Float(3.0),
            Float(2048.0),
            Float(1733.0),
            Float(9.0),
        ];
        assert_wire(&Reply::Tick(tick), "/tick", wire);
    }

    #[test]
    fn a_tick_has_four_values() {
        let three = vec![Int(1), Int(-1), Float(1.0), Float(0.0), Float(0.0)];
        assert_eq!(
            decode_error::<Reply>("/tick", three),
            DecodeErrorKind::Missing {
                index: 5,
                expected: "a float"
            }
        );
    }
}
