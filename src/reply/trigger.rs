//! The trigger notification: scsynth sends `/tr` to every registered client
//! when a `SendTrig` unit generator triggers.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::NodeId;

/// The arguments of `/tr`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tr {
    /// The synth of the `SendTrig`.
    pub node: NodeId,
    /// The ID that the `SendTrig` gives.
    pub id: i32,
    /// The value that the `SendTrig` gives.
    pub value: f32,
}

impl Payload for Tr {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        writer.int(self.id);
        writer.float(self.value);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            node: NodeId::read(reader)?,
            id: reader.int()?,
            value: reader.float()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int};

    use super::*;
    use crate::Reply;
    use crate::testing::assert_wire;

    #[test]
    fn a_trigger_names_its_synth_id_and_value() {
        let trigger = Tr {
            node: NodeId(1000),
            id: 3,
            value: 0.5,
        };
        let wire = vec![Int(1000), Int(3), Float(0.5)];
        assert_wire(&Reply::Tr(trigger), "/tr", wire);
    }
}
