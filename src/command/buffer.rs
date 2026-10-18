//! Buffer commands. The asynchronous ones reply `/done <command> <buffer>`,
//! or `/fail` with the buffer number, and carry a completion message.
//! `/b_gen` is asynchronous, without a completion message.
//!
//! A frame is one sample of each channel. The commands that set or get
//! samples index them across the channels, frame after frame.

use std::fmt;

use super::buffer_fill::BufferFill;
use super::{Completion, read_completion, write_completion};
use crate::codec::{Payload, Reader, Writer, named};
use crate::error::DecodeError;
use crate::types::{BufferId, Element, IndexFill, IndexRange, IndexValue, IndexValues, write_all};

/// The arguments of `/b_alloc`: the buffer takes new memory, of zeros. The
/// server frees the old memory of the buffer.
///
/// scsynth sends no reply when it cannot allocate the memory, also not for
/// a size of 0.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BAlloc {
    /// The buffer.
    pub buffer: BufferId,
    /// The number of frames.
    pub frames: i32,
    /// The number of channels. scsynth takes 1 when the message ends
    /// before it.
    pub channels: i32,
    /// What runs once the buffer is allocated.
    pub completion: Option<Completion>,
    /// The sample rate of the buffer. 0 or less stands for the rate of the
    /// server, which scsynth also takes when the message ends before it.
    pub sample_rate: f32,
}

impl Payload for BAlloc {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.int(self.frames);
        writer.int(self.channels);
        write_completion(writer, self.completion.as_ref());
        writer.float(self.sample_rate);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            frames: reader.int()?,
            channels: reader.int_or(1)?,
            completion: read_completion(reader, true)?,
            sample_rate: reader.float_or(0.0)?,
        })
    }
}

/// The arguments of `/b_allocRead`: the buffer takes new memory, and the
/// frames of a sound file. It takes the channels and the sample rate of
/// the file.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BAllocRead {
    /// The buffer.
    pub buffer: BufferId,
    /// The path of the sound file.
    pub path: String,
    /// The first frame of the file to read. scsynth takes 0 when the
    /// message ends before it.
    pub file_frame: i32,
    /// The number of frames to read. 0 or less reads to the end of the
    /// file, as scsynth does when the message ends before it.
    pub frames: i32,
    /// What runs once the buffer holds the frames.
    pub completion: Option<Completion>,
}

impl Payload for BAllocRead {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.string(&self.path);
        writer.int(self.file_frame);
        writer.int(self.frames);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            path: reader.string()?,
            file_frame: reader.int_or(0)?,
            frames: reader.int_or(0)?,
            completion: read_completion(reader, true)?,
        })
    }
}

/// The arguments of `/b_allocReadChannel`: as `/b_allocRead`, with the
/// channels of the file that the buffer takes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BAllocReadChannel {
    /// The buffer.
    pub buffer: BufferId,
    /// The path of the sound file.
    pub path: String,
    /// The first frame of the file to read.
    pub file_frame: i32,
    /// The number of frames to read. 0 or less reads to the end of the
    /// file.
    pub frames: i32,
    /// The channels of the file.
    pub channels: Channels,
    /// What runs once the buffer holds the frames.
    pub completion: Option<Completion>,
}

impl Payload for BAllocReadChannel {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.string(&self.path);
        writer.int(self.file_frame);
        writer.int(self.frames);
        self.channels.write(writer);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            path: reader.string()?,
            file_frame: reader.int()?,
            frames: reader.int()?,
            channels: Channels::read(reader)?,
            completion: read_completion(reader, false)?,
        })
    }
}

/// The arguments of `/b_read`: frames of a sound file go into the buffer,
/// from one of its frames. The file must have the channels of the buffer.
/// The buffer takes the sample rate of the file.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BRead {
    /// The buffer.
    pub buffer: BufferId,
    /// The path of the sound file.
    pub path: String,
    /// The first frame of the file to read. scsynth takes 0 when the
    /// message ends before it.
    pub file_frame: i32,
    /// The number of frames to read. A negative number reads to the end of
    /// the file, as scsynth does when the message ends before it. The read
    /// stops at the end of the buffer.
    pub frames: i32,
    /// The frame of the buffer that takes the first frame read. scsynth
    /// takes 0 when the message ends before it.
    pub buffer_frame: i32,
    /// Whether the file stays open, for `DiskIn`. scsynth takes `false`
    /// when the message ends before it.
    pub leave_open: bool,
    /// What runs once the buffer holds the frames.
    pub completion: Option<Completion>,
}

impl Payload for BRead {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.string(&self.path);
        writer.int(self.file_frame);
        writer.int(self.frames);
        writer.int(self.buffer_frame);
        writer.flag(self.leave_open);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            path: reader.string()?,
            file_frame: reader.int_or(0)?,
            frames: reader.int_or(-1)?,
            buffer_frame: reader.int_or(0)?,
            leave_open: reader.flag_or(false)?,
            completion: read_completion(reader, true)?,
        })
    }
}

/// The arguments of `/b_readChannel`: as `/b_read`, with the channels of
/// the file that the buffer takes.
///
/// The buffer must have as many channels as the read takes: those of the
/// file, without channels. scsynth 3.14.1 does not check it, and writes
/// past the end of the buffer for more.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BReadChannel {
    /// The buffer.
    pub buffer: BufferId,
    /// The path of the sound file.
    pub path: String,
    /// The first frame of the file to read.
    pub file_frame: i32,
    /// The number of frames to read. A negative number reads to the end of
    /// the file. The read stops at the end of the buffer.
    pub frames: i32,
    /// The frame of the buffer that takes the first frame read.
    pub buffer_frame: i32,
    /// Whether the file stays open, for `DiskIn`.
    pub leave_open: bool,
    /// The channels of the file.
    pub channels: Channels,
    /// What runs once the buffer holds the frames.
    pub completion: Option<Completion>,
}

impl Payload for BReadChannel {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.string(&self.path);
        writer.int(self.file_frame);
        writer.int(self.frames);
        writer.int(self.buffer_frame);
        writer.flag(self.leave_open);
        self.channels.write(writer);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            path: reader.string()?,
            file_frame: reader.int()?,
            frames: reader.int()?,
            buffer_frame: reader.int()?,
            leave_open: reader.flag()?,
            channels: Channels::read(reader)?,
            completion: read_completion(reader, false)?,
        })
    }
}

/// The channels of a sound file that a read takes, in the order that the
/// buffer holds them. No channel at all reads every channel of the file.
///
/// A read takes at most 32 channels: scsynth 3.14.1 writes a 33rd channel
/// past the end of its array of 32.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "Vec<u16>", into = "Vec<u16>")
)]
pub struct Channels(Vec<u16>);

impl Channels {
    /// The most channels that a read takes.
    pub const MAX: usize = 32;

    /// `channels`, when they are at most [`Channels::MAX`].
    #[must_use]
    pub fn new(channels: Vec<u16>) -> Option<Self> {
        (channels.len() <= Self::MAX).then_some(Self(channels))
    }

    /// Every channel of the file.
    #[must_use]
    pub fn all() -> Self {
        Self(Vec::new())
    }

    /// The channels, in order.
    #[must_use]
    pub fn get(&self) -> &[u16] {
        &self.0
    }

    /// Every int that follows, as scsynth reads it.
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let mut channels = Vec::new();
        while let Some(rosc::OscType::Int(_)) = reader.peek() {
            let index = reader.index();
            let channel = reader.int()?;
            let channel = u16::try_from(channel)
                .map_err(|_| reader.invalid(index, format!("{channel} is no channel")))?;
            if channels.len() == Self::MAX {
                return Err(reader.invalid(
                    index,
                    "a 33rd channel, which scsynth 3.14.1 writes past the end of its array",
                ));
            }
            channels.push(channel);
        }
        Ok(Self(channels))
    }

    fn write(&self, writer: &mut Writer) {
        for channel in &self.0 {
            writer.int(i32::from(*channel));
        }
    }
}

impl TryFrom<Vec<u16>> for Channels {
    type Error = TooManyChannels;

    fn try_from(channels: Vec<u16>) -> Result<Self, TooManyChannels> {
        if channels.len() <= Self::MAX {
            Ok(Self(channels))
        } else {
            Err(TooManyChannels(channels))
        }
    }
}

/// More channels than [`Channels::MAX`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TooManyChannels(pub Vec<u16>);

impl fmt::Display for TooManyChannels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} channels, where a read takes at most {}",
            self.0.len(),
            Channels::MAX
        )
    }
}

impl std::error::Error for TooManyChannels {}

impl From<Channels> for Vec<u16> {
    fn from(channels: Channels) -> Self {
        channels.0
    }
}

/// The arguments of `/b_write`: frames of the buffer go into a sound file.
/// Not every sample format goes with every header format.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BWrite {
    /// The buffer.
    pub buffer: BufferId,
    /// The path of the sound file.
    pub path: String,
    /// The header format of the file.
    pub header: HeaderFormat,
    /// The sample format of the file.
    pub sample: SampleFormat,
    /// The number of frames to write. A negative number writes to the end
    /// of the buffer, as scsynth does when the message ends before it.
    pub frames: i32,
    /// The first frame of the buffer to write. scsynth takes 0 when the
    /// message ends before it.
    pub buffer_frame: i32,
    /// Whether the file stays open, for `DiskOut`. scsynth writes the
    /// frames also then: write 0 frames for `DiskOut`. scsynth takes
    /// `false` when the message ends before it.
    pub leave_open: bool,
    /// What runs once the file is written.
    pub completion: Option<Completion>,
}

impl Payload for BWrite {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.string(&self.path);
        writer.string(self.header.name());
        writer.string(self.sample.name());
        writer.int(self.frames);
        writer.int(self.buffer_frame);
        writer.flag(self.leave_open);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            path: reader.string()?,
            header: reader.named("a header format", HeaderFormat::from_name)?,
            sample: reader.named("a sample format", SampleFormat::from_name)?,
            frames: reader.int_or(-1)?,
            buffer_frame: reader.int_or(0)?,
            leave_open: reader.flag_or(false)?,
            completion: read_completion(reader, true)?,
        })
    }
}

named! {
    /// The header format of a sound file of `/b_write`, as libsndfile
    /// writes it. Decoding ignores case, as scsynth does.
    #[non_exhaustive]
    HeaderFormat {
        /// AIFF. scsynth also takes `aifc` for it.
        Aiff = "aiff" | "aifc",
        /// WAV. scsynth also takes `wave` and `riff` for it.
        Wav = "wav" | "wave" | "riff",
        /// WAV with the extensible format.
        WavEx = "wavex",
        /// NeXT and Sun AU. scsynth also takes `sun` for it.
        Next = "next" | "sun",
        /// IRCAM.
        Ircam = "ircam",
        /// Raw samples, without a header.
        Raw = "raw",
        /// Matlab 4.2.
        Mat4 = "mat4",
        /// Matlab 5.0.
        Mat5 = "mat5",
        /// Ensoniq PARIS.
        Paf = "paf",
        /// Amiga IFF and SVX8.
        Svx = "svx",
        /// NIST SPHERE.
        Nist = "nist",
        /// Creative VOC.
        Voc = "voc",
        /// Sonic Foundry 64-bit RIFF and WAV.
        W64 = "w64",
        /// Portable Voice Format.
        Pvf = "pvf",
        /// Fasttracker 2 XI.
        Xi = "xi",
        /// HMM Tool Kit.
        Htk = "htk",
        /// Midi Sample Dump Standard.
        Sds = "sds",
        /// Audio Visual Research.
        Avr = "avr",
        /// Sound Designer II.
        Sd2 = "sd2",
        /// FLAC.
        Flac = "flac",
        /// Ogg, in builds of scsynth whose libsndfile reads Vorbis or Opus.
        Ogg = "ogg",
        /// Core Audio File.
        Caf = "caf",
        /// RF64.
        Rf64 = "rf64",
        /// MPEG, in builds of scsynth whose libsndfile writes MPEG.
        Mpeg = "mpeg",
    }
}

named! {
    /// The sample format of a sound file of `/b_write`. Decoding ignores
    /// case, as scsynth does.
    #[non_exhaustive]
    SampleFormat {
        /// Unsigned 8-bit integers.
        Uint8 = "uint8",
        /// Signed 8-bit integers.
        Int8 = "int8",
        /// Signed 16-bit integers.
        Int16 = "int16",
        /// Signed 24-bit integers.
        Int24 = "int24",
        /// Signed 32-bit integers.
        Int32 = "int32",
        /// 32-bit floats.
        Float = "float",
        /// 64-bit floats.
        Double = "double",
        /// μ-law. scsynth also takes `ulaw` for it.
        Mulaw = "mulaw" | "ulaw",
        /// A-law.
        Alaw = "alaw",
        /// Vorbis, in builds of scsynth whose libsndfile writes it.
        Vorbis = "vorbis",
        /// Opus, in builds of scsynth whose libsndfile writes it.
        Opus = "opus",
        /// MPEG-1 Layer I, in builds of scsynth whose libsndfile writes it.
        Mp1 = "mp1",
        /// MPEG-1 Layer II, in builds of scsynth whose libsndfile writes it.
        Mp2 = "mp2",
        /// MPEG Layer III, in builds of scsynth whose libsndfile writes it.
        Mp3 = "mp3",
    }
}

/// The arguments of `/b_free`, `/b_zero` and `/b_close`. `/b_free` frees
/// the memory of the buffer, and closes its file. `/b_zero` sets every
/// sample of the buffer to 0. `/b_close` closes the file that the buffer
/// keeps open for `DiskOut`, which writes its header.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BFree {
    /// The buffer.
    pub buffer: BufferId,
    /// What runs once the command is done.
    pub completion: Option<Completion>,
}

/// The arguments of `/b_zero`: see [`BFree`].
pub type BZero = BFree;

/// The arguments of `/b_close`: see [`BFree`].
pub type BClose = BFree;

impl Payload for BFree {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        write_completion(writer, self.completion.as_ref());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            completion: read_completion(reader, true)?,
        })
    }
}

/// The arguments of `/b_set`, and of the `/b_set` that answers `/b_get`:
/// samples of the buffer and their values. For a command, scsynth stops at
/// the first index outside the buffer, and fails.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BSet {
    /// The buffer.
    pub buffer: BufferId,
    /// The samples and their values.
    pub samples: Vec<IndexValue>,
}

impl Payload for BSet {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        write_all(writer, &self.samples);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            samples: reader.rest(IndexValue::read)?,
        })
    }
}

/// The arguments of `/b_setn`, and of the `/b_setn` that answers
/// `/b_getn`: ranges of samples of the buffer and their values.
///
/// For a command, keep each range inside the buffer. scsynth 3.14.1 reads
/// only as many values of a range as it has samples inside the buffer,
/// from the first value, and reads the values after them as the next
/// ranges.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BSetn {
    /// The buffer.
    pub buffer: BufferId,
    /// The ranges.
    pub ranges: Vec<IndexValues>,
}

impl Payload for BSetn {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        write_all(writer, &self.ranges);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            ranges: reader.rest(IndexValues::read)?,
        })
    }
}

/// The arguments of `/b_setSampleRate`: the sample rate of the buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BSetSampleRate {
    /// The buffer.
    pub buffer: BufferId,
    /// The sample rate. 0 or less stands for the rate of the server.
    pub sample_rate: f32,
}

impl Payload for BSetSampleRate {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        writer.float(self.sample_rate);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            sample_rate: reader.float()?,
        })
    }
}

/// The arguments of `/b_fill`: ranges of samples of the buffer take one
/// value each. scsynth sets the part of a range inside the buffer.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BFill {
    /// The buffer.
    pub buffer: BufferId,
    /// The ranges.
    pub ranges: Vec<IndexFill>,
}

impl Payload for BFill {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        write_all(writer, &self.ranges);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            ranges: reader.rest(IndexFill::read)?,
        })
    }
}

/// The arguments of `/b_gen`: a fill command writes the buffer. It replies
/// `/done /b_gen <buffer>`, also when the fill leaves the buffer as it is.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BGen {
    /// The buffer.
    pub buffer: BufferId,
    /// The fill command.
    pub fill: BufferFill,
}

impl Payload for BGen {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        self.fill.write(writer);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            fill: BufferFill::read(reader)?,
        })
    }
}

/// The arguments of `/b_query`: scsynth replies `/b_info` for the buffers.
/// See [`crate::reply::buffer::BInfo`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BQuery {
    /// The buffers.
    pub buffers: Vec<BufferId>,
}

impl Payload for BQuery {
    fn write(&self, writer: &mut Writer) {
        for buffer in &self.buffers {
            buffer.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffers: reader.rest(BufferId::read)?,
        })
    }
}

/// The arguments of `/b_get`: scsynth replies `/b_set` with the values of
/// the samples. It fails on an index outside the buffer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BGet {
    /// The buffer.
    pub buffer: BufferId,
    /// The indices of the samples.
    pub samples: Vec<i32>,
}

impl Payload for BGet {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        for sample in &self.samples {
            writer.int(*sample);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            samples: reader.rest(Reader::int)?,
        })
    }
}

/// The arguments of `/b_getn`: scsynth replies `/b_setn` with the values of
/// the ranges. It fails on a range that is empty or not inside the buffer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BGetn {
    /// The buffer.
    pub buffer: BufferId,
    /// The ranges.
    pub ranges: Vec<IndexRange>,
}

impl Payload for BGetn {
    fn write(&self, writer: &mut Writer) {
        self.buffer.write(writer);
        write_all(writer, &self.ranges);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buffer: BufferId::read(reader)?,
            ranges: reader.rest(IndexRange::read)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Blob, Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::command::top_level::SyncId;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode, decode_error};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    fn buffer() -> BufferId {
        BufferId::from(4)
    }

    fn sync() -> Completion {
        Completion::Message(Box::new(Command::Sync(SyncId(1))))
    }

    fn sync_blob() -> rosc::OscType {
        Blob(Command::Sync(SyncId(1)).encode())
    }

    #[test]
    fn an_allocation_writes_every_argument() {
        let alloc = BAlloc {
            buffer: buffer(),
            frames: 1024,
            channels: 2,
            completion: Some(sync()),
            sample_rate: 48_000.0,
        };
        let wire = vec![Int(4), Int(1024), Int(2), sync_blob(), Float(48_000.0)];
        assert_wire(&Command::BAlloc(alloc), "/b_alloc", wire);
        let plain = BAlloc {
            buffer: buffer(),
            frames: 1024,
            channels: 1,
            completion: None,
            sample_rate: 0.0,
        };
        let wire = vec![Int(4), Int(1024), Int(1), Float(0.0)];
        assert_wire(&Command::BAlloc(plain.clone()), "/b_alloc", wire);
        assert_eq!(
            decode::<Command>("/b_alloc", vec![Int(4), Int(1024)]),
            Command::BAlloc(plain.clone())
        );
        let sclang = vec![Int(4), Int(1024), Int(1), Int(0)];
        assert_eq!(
            decode::<Command>("/b_alloc", sclang),
            Command::BAlloc(plain)
        );
    }

    #[test]
    fn an_allocation_refuses_what_scsynth_reads_as_its_channels() {
        // scsynth reads the blob, or the rate, as the number of channels.
        for value in [sync_blob(), Float(48_000.0)] {
            let kind = decode_error::<Command>("/b_alloc", vec![Int(4), Int(1024), value]);
            assert!(
                matches!(kind, DecodeErrorKind::WrongType { index: 2, .. }),
                "{kind:?}"
            );
        }
        let kind = decode_error::<Command>("/b_alloc", vec![Int(-1), Int(1024)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn the_reads_write_every_argument() {
        let read = BAllocRead {
            buffer: buffer(),
            path: "a.wav".to_owned(),
            file_frame: 0,
            frames: 0,
            completion: None,
        };
        let wire = vec![Int(4), text("a.wav"), Int(0), Int(0)];
        assert_wire(&Command::BAllocRead(read.clone()), "/b_allocRead", wire);
        assert_eq!(
            decode::<Command>("/b_allocRead", vec![Int(4), text("a.wav")]),
            Command::BAllocRead(read)
        );
        let read = BRead {
            buffer: buffer(),
            path: "a.wav".to_owned(),
            file_frame: 0,
            frames: -1,
            buffer_frame: 0,
            leave_open: false,
            completion: Some(sync()),
        };
        let wire = vec![
            Int(4),
            text("a.wav"),
            Int(0),
            Int(-1),
            Int(0),
            Int(0),
            sync_blob(),
        ];
        assert_wire(&Command::BRead(read.clone()), "/b_read", wire);
        let short = BRead {
            completion: None,
            ..read
        };
        assert_eq!(
            decode::<Command>("/b_read", vec![Int(4), text("a.wav")]),
            Command::BRead(short)
        );
        let kind = decode_error::<Command>(
            "/b_read",
            vec![Int(4), text("a.wav"), Int(0), Int(-1), Int(0), Int(2)],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 5, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn every_int_before_a_channel_read_completion_is_a_channel() {
        let read = BAllocReadChannel {
            buffer: buffer(),
            path: "a.wav".to_owned(),
            file_frame: 0,
            frames: -1,
            channels: Channels::new(vec![1, 0]).expect("two channels"),
            completion: Some(sync()),
        };
        let wire = vec![
            Int(4),
            text("a.wav"),
            Int(0),
            Int(-1),
            Int(1),
            Int(0),
            sync_blob(),
        ];
        assert_wire(
            &Command::BAllocReadChannel(read.clone()),
            "/b_allocReadChannel",
            wire,
        );
        // The 0 that sclang sends for no completion message is a channel.
        let zero = BAllocReadChannel {
            channels: Channels::new(vec![0]).expect("one channel"),
            completion: None,
            ..read
        };
        assert_eq!(
            decode::<Command>(
                "/b_allocReadChannel",
                vec![Int(4), text("a.wav"), Int(0), Int(-1), Int(0)]
            ),
            Command::BAllocReadChannel(zero)
        );
        let read = BReadChannel {
            buffer: buffer(),
            path: "a.wav".to_owned(),
            file_frame: 0,
            frames: -1,
            buffer_frame: 8,
            leave_open: true,
            channels: Channels::all(),
            completion: None,
        };
        let wire = vec![Int(4), text("a.wav"), Int(0), Int(-1), Int(8), Int(1)];
        assert_wire(&Command::BReadChannel(read), "/b_readChannel", wire);
    }

    #[test]
    fn a_read_takes_at_most_32_channels() {
        assert!(Channels::new(vec![0; Channels::MAX]).is_some());
        assert_eq!(Channels::new(vec![0; Channels::MAX + 1]), None);
        let mut wire = vec![Int(4), text("a.wav"), Int(0), Int(-1)];
        wire.extend((0..33).map(Int));
        let kind = decode_error::<Command>("/b_allocReadChannel", wire);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 36, .. }),
            "{kind:?}"
        );
        let wire = vec![Int(4), text("a.wav"), Int(0), Int(-1), Int(-1)];
        let kind = decode_error::<Command>("/b_allocReadChannel", wire);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 4, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn a_write_names_its_formats() {
        let write = BWrite {
            buffer: buffer(),
            path: "a.wav".to_owned(),
            header: HeaderFormat::Wav,
            sample: SampleFormat::Int24,
            frames: -1,
            buffer_frame: 0,
            leave_open: false,
            completion: None,
        };
        let wire = vec![
            Int(4),
            text("a.wav"),
            text("wav"),
            text("int24"),
            Int(-1),
            Int(0),
            Int(0),
        ];
        assert_wire(&Command::BWrite(write.clone()), "/b_write", wire);
        for (header, sample) in [("WAV", "Int24"), ("riff", "INT24"), ("Wave", "int24")] {
            assert_eq!(
                decode::<Command>(
                    "/b_write",
                    vec![Int(4), text("a.wav"), text(header), text(sample)]
                ),
                Command::BWrite(write.clone()),
                "{header} {sample}"
            );
        }
        let kind = decode_error::<Command>(
            "/b_write",
            vec![Int(4), text("a.wav"), text("mp4"), text("int24")],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn every_format_decodes_from_its_name() {
        for header in [
            HeaderFormat::Aiff,
            HeaderFormat::Wav,
            HeaderFormat::WavEx,
            HeaderFormat::Next,
            HeaderFormat::Ircam,
            HeaderFormat::Raw,
            HeaderFormat::Mat4,
            HeaderFormat::Mat5,
            HeaderFormat::Paf,
            HeaderFormat::Svx,
            HeaderFormat::Nist,
            HeaderFormat::Voc,
            HeaderFormat::W64,
            HeaderFormat::Pvf,
            HeaderFormat::Xi,
            HeaderFormat::Htk,
            HeaderFormat::Sds,
            HeaderFormat::Avr,
            HeaderFormat::Sd2,
            HeaderFormat::Flac,
            HeaderFormat::Ogg,
            HeaderFormat::Caf,
            HeaderFormat::Rf64,
            HeaderFormat::Mpeg,
        ] {
            assert_eq!(HeaderFormat::from_name(header.name()), Some(header));
        }
        for sample in [
            SampleFormat::Uint8,
            SampleFormat::Int8,
            SampleFormat::Int16,
            SampleFormat::Int24,
            SampleFormat::Int32,
            SampleFormat::Float,
            SampleFormat::Double,
            SampleFormat::Mulaw,
            SampleFormat::Alaw,
            SampleFormat::Vorbis,
            SampleFormat::Opus,
            SampleFormat::Mp1,
            SampleFormat::Mp2,
            SampleFormat::Mp3,
        ] {
            assert_eq!(SampleFormat::from_name(sample.name()), Some(sample));
        }
        assert_eq!(HeaderFormat::from_name("AIFC"), Some(HeaderFormat::Aiff));
        assert_eq!(HeaderFormat::from_name("Sun"), Some(HeaderFormat::Next));
        assert_eq!(SampleFormat::from_name("ulaw"), Some(SampleFormat::Mulaw));
        assert_eq!(SampleFormat::from_name(""), None);
    }

    #[test]
    fn a_free_zero_or_close_takes_a_completion_message() {
        let free = BFree {
            buffer: buffer(),
            completion: None,
        };
        assert_wire(&Command::BFree(free.clone()), "/b_free", vec![Int(4)]);
        assert_wire(&Command::BZero(free.clone()), "/b_zero", vec![Int(4)]);
        assert_eq!(
            decode::<Command>("/b_close", vec![Int(4), Int(0)]),
            Command::BClose(free)
        );
        let zero = BFree {
            buffer: buffer(),
            completion: Some(sync()),
        };
        assert_wire(&Command::BZero(zero), "/b_zero", vec![Int(4), sync_blob()]);
    }

    #[test]
    fn the_sample_commands_have_their_wire_form() {
        let set = BSet {
            buffer: buffer(),
            samples: vec![IndexValue {
                index: 3,
                value: 0.5,
            }],
        };
        assert_wire(
            &Command::BSet(set),
            "/b_set",
            vec![Int(4), Int(3), Float(0.5)],
        );
        let setn = BSetn {
            buffer: buffer(),
            ranges: vec![IndexValues {
                start: 3,
                values: vec![0.5, 0.25],
            }],
        };
        let wire = vec![Int(4), Int(3), Int(2), Float(0.5), Float(0.25)];
        assert_wire(&Command::BSetn(setn), "/b_setn", wire);
        let fill = BFill {
            buffer: buffer(),
            ranges: vec![IndexFill {
                start: 3,
                count: 8,
                value: 0.5,
            }],
        };
        let wire = vec![Int(4), Int(3), Int(8), Float(0.5)];
        assert_wire(&Command::BFill(fill), "/b_fill", wire);
        let rate = BSetSampleRate {
            buffer: buffer(),
            sample_rate: 44_100.0,
        };
        let wire = vec![Int(4), Float(44_100.0)];
        assert_wire(&Command::BSetSampleRate(rate), "/b_setSampleRate", wire);
        let query = BQuery {
            buffers: vec![buffer(), BufferId::from(5)],
        };
        assert_wire(&Command::BQuery(query), "/b_query", vec![Int(4), Int(5)]);
        let get = BGet {
            buffer: buffer(),
            samples: vec![3, 4],
        };
        assert_wire(&Command::BGet(get), "/b_get", vec![Int(4), Int(3), Int(4)]);
        let getn = BGetn {
            buffer: buffer(),
            ranges: vec![IndexRange { start: 3, count: 2 }],
        };
        assert_wire(
            &Command::BGetn(getn),
            "/b_getn",
            vec![Int(4), Int(3), Int(2)],
        );
    }

    #[test]
    fn a_sample_value_is_a_float() {
        assert_eq!(
            decode_error::<Command>("/b_set", vec![Int(4), Int(3), Int(1)]),
            DecodeErrorKind::WrongType {
                index: 2,
                expected: "a float",
                found: 'i'
            }
        );
    }
}
