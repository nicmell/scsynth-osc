//! Unit generator commands.

use super::top_level::read_plugin_args;
use crate::argument::Argument;
use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::NodeId;

/// The arguments of `/u_cmd`: a command that a plugin defines for a unit
/// generator of a synth. scsynth reads the synth, the unit and the name,
/// and hands the rest to the unit generator. A synth that has not run yet
/// runs the command once it starts.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UCmd {
    /// The synth.
    pub synth: NodeId,
    /// The index of the unit generator in the synth definition.
    pub unit: i32,
    /// The name of the command, as the plugin registered it.
    pub name: String,
    /// The arguments for the unit generator.
    pub args: Vec<Argument>,
}

impl Payload for UCmd {
    fn write(&self, writer: &mut Writer) {
        self.synth.write(writer);
        writer.int(self.unit);
        writer.string(&self.name);
        for arg in &self.args {
            writer.push(arg.to_osc());
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            synth: NodeId::read(reader)?,
            unit: reader.int()?,
            name: reader.string()?,
            args: read_plugin_args(reader)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::testing::assert_wire;

    #[test]
    fn a_unit_command_names_its_synth_unit_and_command() {
        let command = UCmd {
            synth: NodeId(1000),
            unit: 2,
            name: "setPos".to_owned(),
            args: vec![Argument::Float(0.5)],
        };
        let wire = vec![Int(1000), Int(2), Text("setPos".to_owned()), Float(0.5)];
        assert_wire(&Command::UCmd(command), "/u_cmd", wire);
    }
}
