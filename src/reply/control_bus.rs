//! The replies of the control bus commands: `/c_get` and `/c_getn` answer
//! with the `/c_set` and `/c_setn` that would set the values they read.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{Element, IndexValue, IndexValues, write_all};

/// The arguments of the `/c_set` that answers `/c_get`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CSet {
    /// The buses and their values.
    pub buses: Vec<IndexValue>,
}

impl Payload for CSet {
    fn write(&self, writer: &mut Writer) {
        write_all(writer, &self.buses);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buses: reader.rest(IndexValue::read)?,
        })
    }
}

/// The arguments of the `/c_setn` that answers `/c_getn`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CSetn {
    /// The ranges and their values.
    pub ranges: Vec<IndexValues>,
}

impl Payload for CSetn {
    fn write(&self, writer: &mut Writer) {
        write_all(writer, &self.ranges);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            ranges: reader.rest(IndexValues::read)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn the_bus_answers_carry_floats() {
        let set = CSet {
            buses: vec![IndexValue {
                index: 4,
                value: 0.5,
            }],
        };
        assert_wire(&Reply::CSet(set), "/c_set", vec![Int(4), Float(0.5)]);
        let setn = CSetn {
            ranges: vec![IndexValues {
                start: 4,
                values: vec![0.5, 0.25],
            }],
        };
        let wire = vec![Int(4), Int(2), Float(0.5), Float(0.25)];
        assert_wire(&Reply::CSetn(setn), "/c_setn", wire);
        assert_eq!(
            decode_error::<Reply>("/c_set", vec![Int(4), Int(1)]),
            DecodeErrorKind::WrongType {
                index: 1,
                expected: "a float",
                found: 'i'
            }
        );
    }
}
