//! The messages of the clock of an sc-app3 session: the pong of the pump,
//! and the replies of the clock synth of the session, which scsynth sends
//! with `SendReply` to every registered client.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::packet::Timetag;
use crate::types::NodeId;

/// The arguments of `/clock.pong`: the answer of the pump to a
/// `/clock.ping`, with the clock of the session.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClockPong {
    /// The `sent_ms` of the ping, as it came.
    pub sent_ms: f64,
    /// The time of the bridge when the pump answered.
    pub bridge: Timetag,
    /// The beat of the pump, in milliseconds: the page pings once per beat.
    pub beat_ms: i32,
    /// The OSC time of scsynth minus the time of the bridge, in
    /// milliseconds.
    pub delta_ms: f64,
    /// The smallest round trip of the latest marks between the pump and
    /// scsynth, in milliseconds.
    pub scsynth_rtt_ms: f64,
    /// The clock synth of the timeline: 0 before its first tick, then one
    /// more for each new clock.
    pub epoch: i32,
    /// The OSC time of the sample `x0`.
    pub t0: Timetag,
    /// A sample of the timeline.
    pub x0: i64,
    /// The samples of the timeline per second of OSC time. 0 while the pump
    /// knows no map.
    pub rate: f64,
}

impl Payload for ClockPong {
    fn write(&self, writer: &mut Writer) {
        writer.double(self.sent_ms);
        writer.timetag(self.bridge);
        writer.int(self.beat_ms);
        writer.double(self.delta_ms);
        writer.double(self.scsynth_rtt_ms);
        writer.int(self.epoch);
        writer.timetag(self.t0);
        writer.long(self.x0);
        writer.double(self.rate);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            sent_ms: reader.double()?,
            bridge: reader.timetag()?,
            beat_ms: reader.int()?,
            delta_ms: reader.double()?,
            scsynth_rtt_ms: reader.double()?,
            epoch: reader.int()?,
            t0: reader.timetag()?,
            x0: reader.long()?,
            rate: reader.double()?,
        })
    }
}

/// The arguments of `/tick`, which the clock synth sends every few blocks.
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
    pub at: f32,
}

/// The arguments of `/mark`, which the clock synth sends in the block that
/// runs an `/n_set` of its control `t_mark`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mark {
    /// The clock synth.
    pub node: NodeId,
    /// The reply id of its `SendReply`: −1.
    pub reply: i32,
    /// The first sample of that block, modulo 2^24.
    pub at: f32,
    /// The value of `t_mark`.
    pub id: f32,
}

/// The arguments of `/click`, which the clock synth sends at the sample of a
/// click on its probe bus.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Click {
    /// The clock synth.
    pub node: NodeId,
    /// The reply id of its `SendReply`: −1.
    pub reply: i32,
    /// The sample of the click, modulo 2^24.
    pub sample: f32,
    /// The height of the click, which names it.
    pub id: f32,
}

impl Payload for Tick {
    fn write(&self, writer: &mut Writer) {
        write_reply(writer, self.node, self.reply, [self.count, self.at]);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let (node, reply, [count, at]) = read_reply(reader)?;
        Ok(Self {
            node,
            reply,
            count,
            at,
        })
    }
}

impl Payload for Mark {
    fn write(&self, writer: &mut Writer) {
        write_reply(writer, self.node, self.reply, [self.at, self.id]);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let (node, reply, [at, id]) = read_reply(reader)?;
        Ok(Self {
            node,
            reply,
            at,
            id,
        })
    }
}

impl Payload for Click {
    fn write(&self, writer: &mut Writer) {
        write_reply(writer, self.node, self.reply, [self.sample, self.id]);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let (node, reply, [sample, id]) = read_reply(reader)?;
        Ok(Self {
            node,
            reply,
            sample,
            id,
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
    use rosc::OscTime;
    use rosc::OscType::{Double, Float, Int, Long, Time};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn a_pong_carries_the_clock_of_the_session() {
        let pong = ClockPong {
            sent_ms: 1_792_605_767_123.25,
            bridge: Timetag {
                seconds: 4_001_594_567,
                fraction: 1 << 31,
            },
            beat_ms: 1000,
            delta_ms: -0.03,
            scsynth_rtt_ms: 0.1,
            epoch: 1,
            t0: Timetag {
                seconds: 4_001_594_566,
                fraction: 0,
            },
            x0: 5_000_000_000,
            rate: 48_000.5,
        };
        let wire = vec![
            Double(1_792_605_767_123.25),
            Time(OscTime {
                seconds: 4_001_594_567,
                fractional: 1 << 31,
            }),
            Int(1000),
            Double(-0.03),
            Double(0.1),
            Int(1),
            Time(OscTime {
                seconds: 4_001_594_566,
                fractional: 0,
            }),
            Long(5_000_000_000),
            Double(48_000.5),
        ];
        assert_wire(&Reply::ClockPong(pong), "/clock.pong", wire);
    }

    #[test]
    fn the_replies_of_the_clock_synth_have_their_wire_form() {
        let node = NodeId(67_174_401);
        let tick = Tick {
            node,
            reply: -1,
            count: 3.0,
            at: 2048.0,
        };
        let wire = |a, b| vec![Int(67_174_401), Int(-1), Float(a), Float(b)];
        assert_wire(&Reply::Tick(tick), "/tick", wire(3.0, 2048.0));
        let mark = Mark {
            node,
            reply: -1,
            at: 4096.0,
            id: 7.0,
        };
        assert_wire(&Reply::Mark(mark), "/mark", wire(4096.0, 7.0));
        let click = Click {
            node,
            reply: -1,
            sample: 4133.0,
            id: 9.0,
        };
        assert_wire(&Reply::Click(click), "/click", wire(4133.0, 9.0));
    }

    #[test]
    fn a_tick_has_two_values() {
        assert_eq!(
            decode_error::<Reply>("/tick", vec![Int(1), Int(-1), Float(1.0)]),
            DecodeErrorKind::Missing {
                index: 3,
                expected: "a float"
            }
        );
    }
}
