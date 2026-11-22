//! The answers of sc-app3's pump about the tempo of a session: the tempo
//! that a page reads, and each change of it, which every page gets.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/getTempo.success` and `/tempoChange`: the tempo of a
/// session, as its cycles per second and the cycle at one server time.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tempo {
    /// The cycles per second.
    pub cps: f64,
    /// The cycle at `server_ms`.
    pub cycle: f64,
    /// Server time, in milliseconds since the Unix epoch.
    pub server_ms: f64,
}

impl Payload for Tempo {
    fn write(&self, writer: &mut Writer) {
        writer.double(self.cps);
        writer.double(self.cycle);
        writer.double(self.server_ms);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            cps: reader.double()?,
            cycle: reader.double()?,
            server_ms: reader.double()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Double, Float};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn the_answers_of_the_tempo_have_their_wire_form() {
        let tempo = Tempo {
            cps: 0.5625,
            cycle: 12.0,
            server_ms: 1_792_605_767_324.5,
        };
        let wire = vec![Double(0.5625), Double(12.0), Double(1_792_605_767_324.5)];
        assert_wire(
            &Reply::GetTempoSuccess(tempo),
            "/getTempo.success",
            wire.clone(),
        );
        assert_wire(&Reply::TempoChange(tempo), "/tempoChange", wire);
    }

    #[test]
    fn a_tempo_is_three_doubles() {
        assert_eq!(
            decode_error::<Reply>("/tempoChange", vec![Double(0.5), Double(0.0), Float(1.0)]),
            DecodeErrorKind::WrongType {
                index: 2,
                expected: "a double",
                found: 'f'
            }
        );
    }
}
