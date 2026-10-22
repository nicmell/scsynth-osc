//! The messages of sc-app3's pump about the streams of a session: the
//! answers of an open, and the frames that the holders of a stream get.

use uuid::Uuid;

use crate::codec::{Payload, Reader, Writer};
use crate::command::custom::{read_uuid, write_uuid};
use crate::error::DecodeError;

/// The arguments of `/stream.open.success`: the stream records its bus, and
/// the page gets its frames in `/stream.frames`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamOpened {
    /// The session.
    pub session: Uuid,
    /// The stream.
    pub stream: Uuid,
    /// The channels of each frame.
    pub channels: i32,
}

impl Payload for StreamOpened {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.stream);
        writer.int(self.channels);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            stream: read_uuid(reader)?,
            channels: reader.int()?,
        })
    }
}

/// The arguments of `/stream.frames`, which each holder of a stream gets:
/// frames that follow one another on the timeline of the session.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamFrames {
    /// The stream.
    pub stream: Uuid,
    /// The frame of the timeline of the first frame.
    pub first: f64,
    /// The samples: little-endian `f32`, channels interleaved.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub samples: Vec<u8>,
}

impl Payload for StreamFrames {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.stream);
        writer.double(self.first);
        writer.blob(self.samples.clone());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            stream: read_uuid(reader)?,
            first: reader.double()?,
            samples: reader.blob()?,
        })
    }
}

/// The arguments of `/stream.open.failure`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamFailure {
    /// The session.
    pub session: Uuid,
    /// The stream.
    pub stream: Uuid,
    /// Why the open failed.
    pub reason: String,
}

impl Payload for StreamFailure {
    fn write(&self, writer: &mut Writer) {
        write_uuid(writer, self.session);
        write_uuid(writer, self.stream);
        writer.string(&self.reason);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            session: read_uuid(reader)?,
            stream: read_uuid(reader)?,
            reason: reader.string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Blob, Double, Int, String as Text};

    use super::*;
    use crate::Reply;
    use crate::testing::assert_wire;

    const SESSION: &str = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    const STREAM: &str = "7d1e0c4a-2b3f-4a5e-9c8d-6f7a8b9c0d1e";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_a_stream_open_have_their_wire_form() {
        let opened = StreamOpened {
            session: uuid(SESSION),
            stream: uuid(STREAM),
            channels: 2,
        };
        assert_wire(
            &Reply::StreamOpenSuccess(opened),
            "/stream.open.success",
            vec![text(SESSION), text(STREAM), Int(2)],
        );
        let failure = StreamFailure {
            session: uuid(SESSION),
            stream: uuid(STREAM),
            reason: "Channels must be 1 to 8.".to_owned(),
        };
        assert_wire(
            &Reply::StreamOpenFailure(failure),
            "/stream.open.failure",
            vec![
                text(SESSION),
                text(STREAM),
                text("Channels must be 1 to 8."),
            ],
        );
    }

    #[test]
    fn the_frames_of_a_stream_have_their_wire_form() {
        let samples: Vec<u8> = [0.5_f32, -1.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let frames = StreamFrames {
            stream: uuid(STREAM),
            first: 16_777_216.0 + 1024.0,
            samples: samples.clone(),
        };
        assert_wire(
            &Reply::StreamFrames(frames),
            "/stream.frames",
            vec![text(STREAM), Double(16_778_240.0), Blob(samples)],
        );
    }
}
