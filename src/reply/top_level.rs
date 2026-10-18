//! The replies of the top-level commands. `/synced` carries the
//! [`SyncId`](crate::command::top_level::SyncId) of its `/sync`.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/status.reply`, the load of the server.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatusReply {
    /// The number of unit generators.
    pub ugens: i32,
    /// The number of synths.
    pub synths: i32,
    /// The number of groups, the root group included.
    pub groups: i32,
    /// The number of loaded synth definitions.
    pub synthdefs: i32,
    /// The average CPU usage of the signal processing, in percent.
    pub avg_cpu: f32,
    /// The peak CPU usage of the signal processing, in percent.
    pub peak_cpu: f32,
    /// The nominal sample rate, in Hz.
    pub nominal_sample_rate: f64,
    /// The actual sample rate, in Hz.
    pub actual_sample_rate: f64,
}

impl Payload for StatusReply {
    fn write(&self, writer: &mut Writer) {
        writer.int(1);
        writer.int(self.ugens);
        writer.int(self.synths);
        writer.int(self.groups);
        writer.int(self.synthdefs);
        writer.float(self.avg_cpu);
        writer.float(self.peak_cpu);
        writer.double(self.nominal_sample_rate);
        writer.double(self.actual_sample_rate);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.coded("1, which leads every status", |code| {
            (code == 1).then_some(())
        })?;
        Ok(Self {
            ugens: reader.int()?,
            synths: reader.int()?,
            groups: reader.int()?,
            synthdefs: reader.int()?,
            avg_cpu: reader.float()?,
            peak_cpu: reader.float()?,
            nominal_sample_rate: reader.double()?,
            actual_sample_rate: reader.double()?,
        })
    }
}

/// The arguments of `/version.reply`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VersionReply {
    /// The program: `scsynth`.
    pub program: String,
    /// The major version, as sclang's `Main.scVersionMajor`.
    pub major: i32,
    /// The minor version, as sclang's `Main.scVersionMinor`.
    pub minor: i32,
    /// The patch, with its leading dot, such as `.1`.
    pub patch: String,
    /// The git branch or tag. Builds without git send an empty string.
    pub branch: String,
    /// The abbreviated git commit hash, of a length that git chooses.
    /// Builds without git send `na`.
    pub commit: String,
}

impl Payload for VersionReply {
    fn write(&self, writer: &mut Writer) {
        writer.string(&self.program);
        writer.int(self.major);
        writer.int(self.minor);
        writer.string(&self.patch);
        writer.string(&self.branch);
        writer.string(&self.commit);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            program: reader.string()?,
            major: reader.int()?,
            minor: reader.int()?,
            patch: reader.string()?,
            branch: reader.string()?,
            commit: reader.string()?,
        })
    }
}

/// The arguments of `/rtMemoryStatus.reply`, the real-time memory of the
/// server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RtMemoryStatusReply {
    /// The free real-time memory, in bytes.
    pub free: i32,
    /// The largest free block, in bytes.
    pub largest_block: i32,
}

impl Payload for RtMemoryStatusReply {
    fn write(&self, writer: &mut Writer) {
        writer.int(self.free);
        writer.int(self.largest_block);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            free: reader.int()?,
            largest_block: reader.int()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Double, Float, Int, String as Text};

    use super::*;
    use crate::Reply;
    use crate::command::top_level::SyncId;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    fn status() -> StatusReply {
        StatusReply {
            ugens: 5,
            synths: 1,
            groups: 2,
            synthdefs: 3,
            avg_cpu: 0.5,
            peak_cpu: 1.5,
            nominal_sample_rate: 48_000.0,
            actual_sample_rate: 47_999.5,
        }
    }

    #[test]
    fn the_status_has_the_types_that_scsynth_sends() {
        let wire = vec![
            Int(1),
            Int(5),
            Int(1),
            Int(2),
            Int(3),
            Float(0.5),
            Float(1.5),
            Double(48_000.0),
            Double(47_999.5),
        ];
        assert_wire(&Reply::StatusReply(status()), "/status.reply", wire);
    }

    #[test]
    fn a_status_starts_with_1() {
        let wire = vec![
            Int(0),
            Int(5),
            Int(1),
            Int(2),
            Int(3),
            Float(0.5),
            Float(1.5),
            Double(48_000.0),
            Double(47_999.5),
        ];
        let kind = decode_error::<Reply>("/status.reply", wire);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn the_other_top_level_replies_have_their_wire_form() {
        let version = VersionReply {
            program: "scsynth".to_owned(),
            major: 3,
            minor: 14,
            patch: ".1".to_owned(),
            branch: "Version-3.14.1".to_owned(),
            commit: "426edf6d8".to_owned(),
        };
        let wire = vec![
            Text("scsynth".to_owned()),
            Int(3),
            Int(14),
            Text(".1".to_owned()),
            Text("Version-3.14.1".to_owned()),
            Text("426edf6d8".to_owned()),
        ];
        assert_wire(&Reply::VersionReply(version), "/version.reply", wire);
        let memory = RtMemoryStatusReply {
            free: 8_000_000,
            largest_block: 7_000_000,
        };
        let wire = vec![Int(8_000_000), Int(7_000_000)];
        assert_wire(
            &Reply::RtMemoryStatusReply(memory),
            "/rtMemoryStatus.reply",
            wire,
        );
        assert_wire(&Reply::Synced(SyncId(7)), "/synced", vec![Int(7)]);
    }
}
