//! Synth commands.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{AddAction, Control, ControlSetting, NewNodeId, NodeId};

use super::node::write_nodes;

/// The arguments of `/s_new`: a synth from a synth definition, placed
/// relative to a target. scsynth sends `/n_go` to registered clients,
/// except for an ID that it picked.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SNew {
    /// The name of the synth definition.
    pub def: String,
    /// The ID of the synth.
    pub id: NewNodeId,
    /// Where the synth goes, relative to the target.
    pub action: AddAction,
    /// The target: a group for [`AddAction::Head`] and [`AddAction::Tail`],
    /// a node otherwise.
    pub target: NodeId,
    /// The initial values of controls, set in order, as `/n_set` sets them.
    pub controls: Vec<ControlSetting>,
}

impl Payload for SNew {
    fn write(&self, writer: &mut Writer) {
        writer.string(&self.def);
        self.id.write(writer);
        writer.int(self.action.code());
        self.target.write(writer);
        for setting in &self.controls {
            setting.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            def: reader.string()?,
            id: NewNodeId::read(reader)?,
            action: AddAction::read(reader)?,
            target: NodeId::read(reader)?,
            controls: reader.rest(ControlSetting::read)?,
        })
    }
}

/// The arguments of `/s_get`: scsynth replies `/n_set` with the value of
/// each control, 0 for a control that the synth does not have. See
/// [`crate::reply::synth::NSet`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SGet {
    /// The synth.
    pub synth: NodeId,
    /// The controls.
    pub controls: Vec<Control>,
}

impl Payload for SGet {
    fn write(&self, writer: &mut Writer) {
        self.synth.write(writer);
        for control in &self.controls {
            control.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            synth: NodeId::read(reader)?,
            controls: reader.rest(Control::read)?,
        })
    }
}

/// The arguments of `/s_getn`: scsynth replies `/n_setn` with the values of
/// each range. See [`crate::reply::synth::NSetn`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SGetn {
    /// The synth.
    pub synth: NodeId,
    /// The ranges.
    pub ranges: Vec<RangeRequest>,
}

/// A range of `/s_getn`: controls from one.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeRequest {
    /// The first control of the range.
    pub control: Control,
    /// The number of controls. A negative count is a [trap](crate#traps).
    pub count: i32,
}

impl Payload for SGetn {
    fn write(&self, writer: &mut Writer) {
        self.synth.write(writer);
        for range in &self.ranges {
            range.control.write(writer);
            writer.int(range.count);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let synth = NodeId::read(reader)?;
        let ranges = reader.rest(|reader| {
            Ok(RangeRequest {
                control: Control::read(reader)?,
                count: reader.int()?,
            })
        })?;
        Ok(Self { synth, ranges })
    }
}

/// The arguments of `/s_noid`: the synths keep running under IDs that the
/// server picks, and their own IDs are free again. scsynth sends no
/// notification for it, nor for the synths afterwards.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SNoid {
    /// The synths.
    pub synths: Vec<NodeId>,
}

impl Payload for SNoid {
    fn write(&self, writer: &mut Writer) {
        write_nodes(writer, &self.synths);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            synths: reader.rest(NodeId::read)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscArray;
    use rosc::OscType::{Array, Double, Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};
    use crate::types::{ArrayElement, BusMapping, ControlValue};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    fn header() -> Vec<rosc::OscType> {
        vec![text("default"), Int(1000), Int(0), Int(1)]
    }

    fn synth(controls: Vec<ControlSetting>) -> Command {
        Command::SNew(SNew {
            def: "default".to_owned(),
            id: NewNodeId::new(1000).expect("an ID of 0 or more"),
            action: AddAction::Head,
            target: NodeId(1),
            controls,
        })
    }

    #[test]
    fn a_synth_takes_every_form_of_control_value() {
        let controls = vec![
            ControlSetting::new("freq", 440.0),
            ControlSetting::new(1, 2),
            ControlSetting::new("amp", BusMapping::Control(3)),
            ControlSetting::new("in", BusMapping::Audio(4)),
            ControlSetting::new(
                "partials",
                ControlValue::Array(vec![
                    ArrayElement::Float(0.5),
                    ArrayElement::Int(1),
                    ArrayElement::Bus(BusMapping::Control(9)),
                ]),
            ),
        ];
        let mut wire = header();
        wire.extend([
            text("freq"),
            Float(440.0),
            Int(1),
            Int(2),
            text("amp"),
            text("c3"),
            text("in"),
            text("a4"),
            text("partials"),
            Array(OscArray {
                content: vec![Float(0.5), Int(1), text("c9")],
            }),
        ]);
        assert_wire(&synth(controls), "/s_new", wire);
        assert_wire(&synth(vec![]), "/s_new", header());
    }

    #[test]
    fn a_synth_of_the_server_has_id_minus_1() {
        let new = SNew {
            def: "default".to_owned(),
            id: NewNodeId::AUTO,
            action: AddAction::Replace,
            target: NodeId(1000),
            controls: vec![],
        };
        let wire = vec![text("default"), Int(-1), Int(4), Int(1000)];
        assert_wire(&Command::SNew(new), "/s_new", wire);
        let kind =
            decode_error::<Command>("/s_new", vec![text("default"), Int(-2), Int(0), Int(1)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
        let kind =
            decode_error::<Command>("/s_new", vec![text("default"), Int(1000), Int(5), Int(1)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn a_control_value_that_scsynth_misreads_is_refused() {
        let decode = |value| {
            let mut wire = header();
            wire.extend([text("freq"), value]);
            decode_error::<Command>("/s_new", wire)
        };
        assert_eq!(
            decode(Double(440.0)),
            DecodeErrorKind::WrongType {
                index: 5,
                expected: "a control value: an int, a float, a bus mapping or an array",
                found: 'd'
            }
        );
        for value in [
            text("cx"),
            text("c"),
            Array(OscArray {
                content: vec![Float(1.0), Double(2.0)],
            }),
            Array(OscArray {
                content: vec![Array(OscArray {
                    content: vec![Float(1.0)],
                })],
            }),
            Array(OscArray {
                content: vec![text("b1")],
            }),
        ] {
            let kind = decode(value.clone());
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 5, .. }),
                "{value:?}: {kind:?}"
            );
        }
    }

    #[test]
    fn a_name_ends_at_its_first_nul() {
        let new = SNew {
            def: "default\0junk".to_owned(),
            id: NewNodeId::AUTO,
            action: AddAction::Head,
            target: NodeId(1),
            controls: vec![ControlSetting::new("fr\0eq", 1.0)],
        };
        let wire = vec![
            text("default"),
            Int(-1),
            Int(0),
            Int(1),
            text("fr"),
            Float(1.0),
        ];
        assert_eq!(
            crate::codec::sealed::Codec::to_osc(&Command::SNew(new)).args,
            wire
        );
    }

    #[test]
    fn the_queries_have_their_wire_form() {
        let get = SGet {
            synth: NodeId(1000),
            controls: vec![Control::from("freq"), Control::Index(2)],
        };
        let wire = vec![Int(1000), text("freq"), Int(2)];
        assert_wire(&Command::SGet(get), "/s_get", wire);
        let getn = SGetn {
            synth: NodeId(1000),
            ranges: vec![RangeRequest {
                control: Control::from("freqs"),
                count: 3,
            }],
        };
        let wire = vec![Int(1000), text("freqs"), Int(3)];
        assert_wire(&Command::SGetn(getn), "/s_getn", wire);
        assert_eq!(
            decode_error::<Command>("/s_getn", vec![Int(1000), text("freqs")]),
            DecodeErrorKind::Missing {
                index: 2,
                expected: "an int"
            }
        );
        let noid = SNoid {
            synths: vec![NodeId(1000), NodeId(1001)],
        };
        assert_wire(&Command::SNoid(noid), "/s_noid", vec![Int(1000), Int(1001)]);
    }
}
