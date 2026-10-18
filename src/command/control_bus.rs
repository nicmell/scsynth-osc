//! Control bus commands.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{Element, IndexFill, IndexRange, IndexValue, IndexValues, Number, write_all};

/// The arguments of `/c_set`: buses and their values. scsynth stops at the
/// first bus that it does not have, and fails.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CSet {
    /// The buses and their values.
    pub buses: Vec<IndexValue<Number>>,
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

/// The arguments of `/c_setn`: ranges of buses and their values. scsynth
/// stops at the first range that is empty or not inside its buses, and
/// fails.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CSetn {
    /// The ranges.
    pub ranges: Vec<IndexValues<Number>>,
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

/// The arguments of `/c_fill`: ranges of buses take one value each.
/// scsynth sets the part of a range inside its buses.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CFill {
    /// The ranges.
    pub ranges: Vec<IndexFill<Number>>,
}

impl Payload for CFill {
    fn write(&self, writer: &mut Writer) {
        write_all(writer, &self.ranges);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            ranges: reader.rest(IndexFill::read)?,
        })
    }
}

/// The arguments of `/c_get`: scsynth replies `/c_set` with the values of
/// the buses. It fails on a bus that it does not have. See
/// [`crate::reply::control_bus::CSet`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CGet {
    /// The buses.
    pub buses: Vec<i32>,
}

impl Payload for CGet {
    fn write(&self, writer: &mut Writer) {
        for bus in &self.buses {
            writer.int(*bus);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            buses: reader.rest(Reader::int)?,
        })
    }
}

/// The arguments of `/c_getn`: scsynth replies `/c_setn` with the values of
/// the ranges. It fails on a range that is empty or not inside its buses.
/// See [`crate::reply::control_bus::CSetn`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CGetn {
    /// The ranges.
    pub ranges: Vec<IndexRange>,
}

impl Payload for CGetn {
    fn write(&self, writer: &mut Writer) {
        write_all(writer, &self.ranges);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            ranges: reader.rest(IndexRange::read)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int};

    use super::*;
    use crate::Command;
    use crate::testing::assert_wire;

    #[test]
    fn the_bus_commands_take_ints_and_floats() {
        let set = CSet {
            buses: vec![
                IndexValue {
                    index: 0,
                    value: Number::Float(0.5),
                },
                IndexValue {
                    index: 1,
                    value: Number::Int(2),
                },
            ],
        };
        let wire = vec![Int(0), Float(0.5), Int(1), Int(2)];
        assert_wire(&Command::CSet(set), "/c_set", wire);
        let setn = CSetn {
            ranges: vec![IndexValues {
                start: 4,
                values: vec![Number::Int(1), Number::Float(0.5)],
            }],
        };
        let wire = vec![Int(4), Int(2), Int(1), Float(0.5)];
        assert_wire(&Command::CSetn(setn), "/c_setn", wire);
        let fill = CFill {
            ranges: vec![IndexFill {
                start: 4,
                count: 8,
                value: Number::Float(0.5),
            }],
        };
        let wire = vec![Int(4), Int(8), Float(0.5)];
        assert_wire(&Command::CFill(fill), "/c_fill", wire);
        let get = CGet { buses: vec![0, 4] };
        assert_wire(&Command::CGet(get), "/c_get", vec![Int(0), Int(4)]);
        let getn = CGetn {
            ranges: vec![IndexRange { start: 4, count: 2 }],
        };
        assert_wire(&Command::CGetn(getn), "/c_getn", vec![Int(4), Int(2)]);
    }
}
