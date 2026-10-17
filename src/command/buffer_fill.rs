//! Buffer fill commands: what `/b_gen` writes into a buffer.
//!
//! The wave fills write a buffer of one channel only, and leave any other
//! buffer as it is.

use super::top_level::read_plugin_args;
use crate::argument::Argument;
use crate::codec::{Reader, Writer};
use crate::error::DecodeError;
use crate::types::BufferId;

/// A fill command of `/b_gen`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum BufferFill {
    /// `sine1`: sine partials at whole multiples of the frequency of the
    /// buffer.
    Sine1 {
        /// How the partials are written.
        flags: WaveFillFlags,
        /// The amplitude of each partial, from the first.
        amplitudes: Vec<f32>,
    },
    /// `sine2`: sine partials at any frequency.
    Sine2 {
        /// How the partials are written.
        flags: WaveFillFlags,
        /// The partials.
        partials: Vec<Partial>,
    },
    /// `sine3`: sine partials at any frequency and phase.
    Sine3 {
        /// How the partials are written.
        flags: WaveFillFlags,
        /// The partials.
        partials: Vec<PhasedPartial>,
    },
    /// `cheby`: a sum of Chebyshev polynomials, for a waveshaper. The
    /// buffer is offset so that its center is 0.
    Cheby {
        /// How the polynomials are written.
        flags: WaveFillFlags,
        /// The amplitude of each polynomial, from the first order.
        amplitudes: Vec<f32>,
    },
    /// `copy`: frames of another buffer. The two buffers must have the same
    /// channels, or nothing is copied.
    Copy {
        /// The first frame of this buffer to write.
        to_frame: i32,
        /// The buffer to read.
        source: BufferId,
        /// The first frame of the source to read.
        from_frame: i32,
        /// The number of frames. A negative number copies as many as the
        /// two buffers hold. scsynth copies nothing when the message ends
        /// before it.
        frames: i32,
    },
    /// A fill command that a plugin defines, such as `normalize`. A name of
    /// another variant decodes as that variant.
    Plugin {
        /// The name of the command, as the plugin registered it.
        name: String,
        /// The arguments for the command.
        args: Vec<Argument>,
    },
}

/// A partial of `sine2`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Partial {
    /// The frequency, in cycles per buffer.
    pub frequency: f32,
    /// The amplitude.
    pub amplitude: f32,
}

/// A partial of `sine3`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhasedPartial {
    /// The frequency, in cycles per buffer.
    pub frequency: f32,
    /// The amplitude.
    pub amplitude: f32,
    /// The phase, in radians.
    pub phase: f32,
}

/// How a wave fill writes the buffer: the bits 1, 2 and 4 of an int.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaveFillFlags {
    /// The peak amplitude becomes 1.
    pub normalize: bool,
    /// The buffer holds the wavetable format, for the interpolating
    /// oscillators.
    pub wavetable: bool,
    /// The buffer is cleared first. Otherwise, the partials add to it.
    pub clear: bool,
}

impl WaveFillFlags {
    const NORMALIZE: i32 = 1;
    const WAVETABLE: i32 = 2;
    const CLEAR: i32 = 4;

    /// The int that travels on the wire.
    pub fn bits(self) -> i32 {
        let bit = |set, bit| if set { bit } else { 0 };
        bit(self.normalize, Self::NORMALIZE)
            | bit(self.wavetable, Self::WAVETABLE)
            | bit(self.clear, Self::CLEAR)
    }

    /// The flags of `bits`, when it holds no other bit.
    pub fn from_bits(bits: i32) -> Option<Self> {
        (bits & !(Self::NORMALIZE | Self::WAVETABLE | Self::CLEAR) == 0).then_some(Self {
            normalize: bits & Self::NORMALIZE != 0,
            wavetable: bits & Self::WAVETABLE != 0,
            clear: bits & Self::CLEAR != 0,
        })
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.coded("wave fill flags, 0 to 7", Self::from_bits)
    }
}

impl BufferFill {
    /// The name of the command.
    pub fn name(&self) -> &str {
        match self {
            Self::Sine1 { .. } => "sine1",
            Self::Sine2 { .. } => "sine2",
            Self::Sine3 { .. } => "sine3",
            Self::Cheby { .. } => "cheby",
            Self::Copy { .. } => "copy",
            Self::Plugin { name, .. } => name,
        }
    }

    pub(crate) fn write(&self, writer: &mut Writer) {
        writer.string(self.name());
        match self {
            Self::Sine1 { flags, amplitudes } | Self::Cheby { flags, amplitudes } => {
                writer.int(flags.bits());
                for amplitude in amplitudes {
                    writer.float(*amplitude);
                }
            }
            Self::Sine2 { flags, partials } => {
                writer.int(flags.bits());
                for partial in partials {
                    writer.float(partial.frequency);
                    writer.float(partial.amplitude);
                }
            }
            Self::Sine3 { flags, partials } => {
                writer.int(flags.bits());
                for partial in partials {
                    writer.float(partial.frequency);
                    writer.float(partial.amplitude);
                    writer.float(partial.phase);
                }
            }
            Self::Copy {
                to_frame,
                source,
                from_frame,
                frames,
            } => {
                writer.int(*to_frame);
                source.write(writer);
                writer.int(*from_frame);
                writer.int(*frames);
            }
            Self::Plugin { args, .. } => {
                for arg in args {
                    writer.push(arg.to_osc());
                }
            }
        }
    }

    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let name = reader.string()?;
        Ok(match name.as_str() {
            "sine1" => Self::Sine1 {
                flags: WaveFillFlags::read(reader)?,
                amplitudes: reader.rest(Reader::float)?,
            },
            "sine2" => Self::Sine2 {
                flags: WaveFillFlags::read(reader)?,
                partials: reader.rest(|reader| {
                    Ok(Partial {
                        frequency: reader.float()?,
                        amplitude: reader.float()?,
                    })
                })?,
            },
            "sine3" => Self::Sine3 {
                flags: WaveFillFlags::read(reader)?,
                partials: reader.rest(|reader| {
                    Ok(PhasedPartial {
                        frequency: reader.float()?,
                        amplitude: reader.float()?,
                        phase: reader.float()?,
                    })
                })?,
            },
            "cheby" => Self::Cheby {
                flags: WaveFillFlags::read(reader)?,
                amplitudes: reader.rest(Reader::float)?,
            },
            "copy" => Self::Copy {
                to_frame: reader.int()?,
                source: BufferId::read(reader)?,
                from_frame: reader.int()?,
                frames: reader.int()?,
            },
            _ => Self::Plugin {
                name,
                args: read_plugin_args(reader)?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::command::buffer::BGen;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    fn generate(fill: BufferFill) -> Command {
        Command::BGen(BGen {
            buffer: BufferId::from(4),
            fill,
        })
    }

    #[test]
    fn the_wave_fills_have_their_wire_form() {
        let flags = WaveFillFlags {
            normalize: true,
            wavetable: true,
            clear: true,
        };
        let sine1 = BufferFill::Sine1 {
            flags,
            amplitudes: vec![1.0, 0.5],
        };
        let wire = vec![Int(4), text("sine1"), Int(7), Float(1.0), Float(0.5)];
        assert_wire(&generate(sine1), "/b_gen", wire);
        let sine2 = BufferFill::Sine2 {
            flags: WaveFillFlags::default(),
            partials: vec![Partial {
                frequency: 1.5,
                amplitude: 0.5,
            }],
        };
        let wire = vec![Int(4), text("sine2"), Int(0), Float(1.5), Float(0.5)];
        assert_wire(&generate(sine2), "/b_gen", wire);
        let sine3 = BufferFill::Sine3 {
            flags: WaveFillFlags {
                clear: true,
                ..WaveFillFlags::default()
            },
            partials: vec![PhasedPartial {
                frequency: 1.0,
                amplitude: 0.5,
                phase: 0.25,
            }],
        };
        let wire = vec![
            Int(4),
            text("sine3"),
            Int(4),
            Float(1.0),
            Float(0.5),
            Float(0.25),
        ];
        assert_wire(&generate(sine3), "/b_gen", wire);
        let cheby = BufferFill::Cheby {
            flags: WaveFillFlags {
                normalize: true,
                ..WaveFillFlags::default()
            },
            amplitudes: vec![1.0],
        };
        let wire = vec![Int(4), text("cheby"), Int(1), Float(1.0)];
        assert_wire(&generate(cheby), "/b_gen", wire);
    }

    #[test]
    fn a_copy_names_its_frames_and_its_source() {
        let copy = BufferFill::Copy {
            to_frame: 0,
            source: BufferId::from(5),
            from_frame: 8,
            frames: -1,
        };
        let wire = vec![Int(4), text("copy"), Int(0), Int(5), Int(8), Int(-1)];
        assert_wire(&generate(copy), "/b_gen", wire);
        assert_eq!(
            decode_error::<Command>("/b_gen", vec![Int(4), text("copy"), Int(0), Int(5), Int(8)]),
            DecodeErrorKind::Missing {
                index: 5,
                expected: "an int"
            }
        );
    }

    #[test]
    fn a_fill_is_refused_where_scsynth_reads_it_otherwise() {
        let kind = decode_error::<Command>("/b_gen", vec![Int(4), text("sine1"), Int(8)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Command>("/b_gen", vec![Int(4), text("sine2"), Int(0), Float(1.0)]),
            DecodeErrorKind::Missing {
                index: 4,
                expected: "a float"
            }
        );
        assert_eq!(
            decode_error::<Command>("/b_gen", vec![Int(4), text("cheby"), Int(0), Int(1)]),
            DecodeErrorKind::WrongType {
                index: 3,
                expected: "a float",
                found: 'i'
            }
        );
    }

    #[test]
    fn another_name_is_a_plugin_fill() {
        let normalize = BufferFill::Plugin {
            name: "normalize".to_owned(),
            args: vec![Argument::Float(0.5)],
        };
        let wire = vec![Int(4), text("normalize"), Float(0.5)];
        assert_wire(&generate(normalize), "/b_gen", wire);
    }
}
