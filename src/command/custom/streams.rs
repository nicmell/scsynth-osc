//! The commands of sc-app3's pump about the streams of a session: a stream
//! records an audio bus into a ring buffer that the pump reads at each tick
//! of the clock of the session.

use uuid::Uuid;

use super::{read_uuid, write_uuid};
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/stream.open`: a stream of a session, and the audio
/// bus that it records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamOpen {
    /// The session.
    pub session: Uuid,
    /// The stream, which the page names.
    pub stream: Uuid,
    /// The first audio bus that the stream records.
    pub bus: i32,
    /// The count of buses from `bus` that the stream records.
    pub channels: i32,
}

impl Payload for StreamOpen {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.stream);
        writer.int(self.bus);
        writer.int(self.channels);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            stream: read_uuid(reader)?,
            bus: reader.int()?,
            channels: reader.int()?,
        })
    }
}

/// The arguments of `/stream.close`: a stream of a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stream {
    /// The session.
    pub session: Uuid,
    /// The stream.
    pub stream: Uuid,
}

impl Payload for Stream {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.stream);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            stream: read_uuid(reader)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const STREAM: &str = "7d1e0c4a-2b3f-4a5e-9c8d-6f7a8b9c0d1e";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_commands_of_the_streams_have_their_wire_form() {
        let open = StreamOpen {
            session: uuid(SESSION),
            stream: uuid(STREAM),
            bus: 256,
            channels: 2,
        };
        assert_wire(
            &Command::StreamOpen(open),
            "/stream.open",
            vec![text(SESSION), text(STREAM), Int(256), Int(2)],
        );
        let close = Stream {
            session: uuid(SESSION),
            stream: uuid(STREAM),
        };
        assert_wire(
            &Command::StreamClose(close),
            "/stream.close",
            vec![text(SESSION), text(STREAM)],
        );
    }

    #[test]
    fn a_stream_is_a_uuid_and_its_bus_an_int() {
        let kind =
            decode_error::<Command>("/stream.close", vec![text(SESSION), text("not a uuid")]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Command>(
                "/stream.open",
                vec![text(SESSION), text(STREAM), Float(256.0), Int(2)]
            ),
            DecodeErrorKind::WrongType {
                index: 2,
                expected: "an int",
                found: 'f'
            }
        );
    }
}
