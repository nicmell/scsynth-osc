//! The commands of sc-app3's pump about the tempo of a session, on which
//! the pump times the sounds of its Dirts: a page reads it, and sets it.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;

/// The arguments of `/setTempo`: the tempo that a page sets for the
/// session.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetTempo {
    /// The cycles per second.
    pub cps: f64,
}

impl Payload for SetTempo {
    fn write(&self, writer: &mut Writer) {
        writer.double(self.cps);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            cps: reader.double()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Double, Float};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn the_commands_of_the_tempo_have_their_wire_form() {
        assert_wire(&Command::GetTempo, "/getTempo", vec![]);
        assert_wire(
            &Command::SetTempo(SetTempo { cps: 0.5625 }),
            "/setTempo",
            vec![Double(0.5625)],
        );
    }

    #[test]
    fn a_tempo_is_a_double() {
        assert_eq!(
            decode_error::<Command>("/setTempo", vec![Float(0.5)]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "a double",
                found: 'f'
            }
        );
    }
}
