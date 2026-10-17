//! The replies of the buffer commands. `/b_get` and `/b_getn` answer with
//! the `/b_set` and `/b_setn` that would set the values they read.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::BufferId;

/// The arguments of the `/b_set` that answers `/b_get`: see
/// [`crate::command::buffer::BSet`].
pub type BSet = crate::command::buffer::BSet;

/// The arguments of the `/b_setn` that answers `/b_getn`: see
/// [`crate::command::buffer::BSetn`].
pub type BSetn = crate::command::buffer::BSetn;

/// The arguments of `/b_info`, the answer to `/b_query`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BInfo {
    /// The buffers, in the order of the query.
    pub buffers: Vec<BufferInfo>,
}

/// A buffer of `/b_info`. A buffer without memory has 0 frames and 0
/// channels.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferInfo {
    /// The buffer, as the query named it.
    pub buffer: BufferId,
    /// The number of frames.
    pub frames: i32,
    /// The number of channels.
    pub channels: i32,
    /// The sample rate.
    pub sample_rate: f32,
}

impl Payload for BInfo {
    fn write(&self, writer: &mut Writer) {
        for info in &self.buffers {
            info.buffer.write(writer);
            writer.int(info.frames);
            writer.int(info.channels);
            writer.float(info.sample_rate);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let buffers = reader.rest(|reader| {
            Ok(BufferInfo {
                buffer: BufferId::read(reader)?,
                frames: reader.int()?,
                channels: reader.int()?,
                sample_rate: reader.float()?,
            })
        })?;
        Ok(Self { buffers })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int};

    use super::*;
    use crate::Reply;
    use crate::testing::assert_wire;
    use crate::types::{IndexValue, IndexValues};

    #[test]
    fn the_buffer_answers_have_their_wire_form() {
        let info = BInfo {
            buffers: vec![
                BufferInfo {
                    buffer: BufferId::from(4),
                    frames: 1024,
                    channels: 2,
                    sample_rate: 48_000.0,
                },
                BufferInfo {
                    buffer: BufferId::from(5),
                    frames: 0,
                    channels: 0,
                    sample_rate: 0.0,
                },
            ],
        };
        let wire = vec![
            Int(4),
            Int(1024),
            Int(2),
            Float(48_000.0),
            Int(5),
            Int(0),
            Int(0),
            Float(0.0),
        ];
        assert_wire(&Reply::BInfo(info), "/b_info", wire);
        let set = BSet {
            buffer: BufferId::from(4),
            samples: vec![IndexValue {
                index: 3,
                value: 0.5,
            }],
        };
        assert_wire(
            &Reply::BSet(set),
            "/b_set",
            vec![Int(4), Int(3), Float(0.5)],
        );
        let setn = BSetn {
            buffer: BufferId::from(4),
            ranges: vec![IndexValues {
                start: 3,
                values: vec![0.5],
            }],
        };
        let wire = vec![Int(4), Int(3), Int(1), Float(0.5)];
        assert_wire(&Reply::BSetn(setn), "/b_setn", wire);
    }
}
