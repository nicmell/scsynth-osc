//! Node commands: they act on synths and on groups. A command that sets or
//! maps the controls of a group acts on every synth in it.
//!
//! scsynth tells registered clients about the changes, such as `/n_end`
//! for `/n_free`: see [`crate::reply::node_notification`].

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{Control, ControlSetting, NodeId, Number, OrderAction};

/// The arguments of `/n_free`: the nodes stop, and the server frees them at
/// once. A node that sounds can click.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NFree {
    /// The nodes.
    pub nodes: Vec<NodeId>,
}

impl Payload for NFree {
    fn write(&self, writer: &mut Writer) {
        write_nodes(writer, &self.nodes);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            nodes: reader.rest(NodeId::read)?,
        })
    }
}

/// The arguments of `/n_run`: nodes turn on or off. A node that is off
/// does not run, and keeps its state. A node that sounds can click.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NRun {
    /// The nodes, each with its flag.
    pub nodes: Vec<NodeRun>,
}

/// A node of `/n_run`, and whether it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeRun {
    /// The node.
    pub node: NodeId,
    /// Whether the node runs.
    pub run: bool,
}

impl Payload for NRun {
    fn write(&self, writer: &mut Writer) {
        for entry in &self.nodes {
            entry.node.write(writer);
            writer.flag(entry.run);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let nodes = reader.rest(|reader| {
            Ok(NodeRun {
                node: NodeId::read(reader)?,
                run: reader.flag()?,
            })
        })?;
        Ok(Self { nodes })
    }
}

/// The arguments of `/n_set`: controls take values, or read buses. A value
/// also undoes the mapping of its control to a bus.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NSet {
    /// The node.
    pub node: NodeId,
    /// The controls and their values, set in order.
    pub controls: Vec<ControlSetting>,
}

impl Payload for NSet {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for setting in &self.controls {
            setting.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            node: NodeId::read(reader)?,
            controls: reader.rest(ControlSetting::read)?,
        })
    }
}

/// The arguments of `/n_setn`: ranges of controls take values.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NSetn {
    /// The node.
    pub node: NodeId,
    /// The ranges.
    pub ranges: Vec<ControlRange>,
}

/// A range of `/n_setn`: a control, and the values for it and for the
/// controls after it.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlRange {
    /// The first control of the range.
    pub control: Control,
    /// The values, one per control.
    pub values: Vec<Number>,
}

impl Payload for NSetn {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for range in &self.ranges {
            range.control.write(writer);
            writer.count(range.values.len());
            for value in &range.values {
                value.write(writer);
            }
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let ranges = reader.rest(|reader| {
            let control = Control::read(reader)?;
            let count = reader.count()?;
            let values = (0..count)
                .map(|_| Number::read(reader))
                .collect::<Result<_, _>>()?;
            Ok(ControlRange { control, values })
        })?;
        Ok(Self { node, ranges })
    }
}

/// The arguments of `/n_fill`: ranges of controls take one value each.
///
/// scsynth 3.14.1 reads a fourth argument for each range, and sets the
/// controls to it, or to 0 when the message ends. The crate writes the
/// value twice, which scsynth reads as the reference means it, and decodes
/// a range only with the same value twice.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NFill {
    /// The node.
    pub node: NodeId,
    /// The ranges.
    pub ranges: Vec<ControlFill>,
}

/// A range of `/n_fill`: controls from one, and the value for them all.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlFill {
    /// The first control of the range.
    pub control: Control,
    /// The number of controls.
    pub count: i32,
    /// The value.
    pub value: Number,
}

impl Payload for NFill {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for range in &self.ranges {
            range.control.write(writer);
            writer.int(range.count);
            range.value.write(writer);
            range.value.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let ranges = reader.rest(|reader| {
            let control = Control::read(reader)?;
            let count = reader.int()?;
            let value = Number::read(reader)?;
            let index = reader.index();
            let again = Number::read(reader)?;
            if !value.same(again) {
                return Err(reader.invalid(
                    index,
                    "a fill value that differs from the one before it, which scsynth sets in its place",
                ));
            }
            Ok(ControlFill {
                control,
                count,
                value,
            })
        })?;
        Ok(Self { node, ranges })
    }
}

/// The arguments of `/n_map` and `/n_mapa`: controls read buses, control
/// buses for `/n_map` and audio buses for `/n_mapa`. A control that reads
/// an audio bus takes the full rate only when its rate is `\ar`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NMap {
    /// The node.
    pub node: NodeId,
    /// The controls and their buses.
    pub mappings: Vec<ControlBus>,
}

/// A control of `/n_map` or `/n_mapa`, and its bus.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlBus {
    /// The control.
    pub control: Control,
    /// The index of the bus. -1, as any negative index, undoes the mapping.
    pub bus: i32,
}

/// The arguments of `/n_mapa`: see [`NMap`].
pub type NMapa = NMap;

impl Payload for NMap {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for mapping in &self.mappings {
            mapping.control.write(writer);
            writer.int(mapping.bus);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let mappings = reader.rest(|reader| {
            Ok(ControlBus {
                control: Control::read(reader)?,
                bus: reader.int()?,
            })
        })?;
        Ok(Self { node, mappings })
    }
}

/// The arguments of `/n_mapn` and `/n_mapan`: ranges of controls read
/// consecutive buses, control buses for `/n_mapn` and audio buses for
/// `/n_mapan`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NMapn {
    /// The node.
    pub node: NodeId,
    /// The ranges and their first buses.
    pub mappings: Vec<ControlBusRange>,
}

/// A range of `/n_mapn` or `/n_mapan`: controls from one, and the bus of
/// the first one.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlBusRange {
    /// The first control of the range.
    pub control: Control,
    /// The bus of the first control. The next control reads the next bus.
    /// -1 undoes the mapping of the whole range. scsynth counts up from
    /// any other negative index, so the later controls of the range read
    /// buses from 0.
    pub bus: i32,
    /// The number of controls.
    pub count: i32,
}

/// The arguments of `/n_mapan`: see [`NMapn`].
pub type NMapan = NMapn;

impl Payload for NMapn {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        for mapping in &self.mappings {
            mapping.control.write(writer);
            writer.int(mapping.bus);
            writer.int(mapping.count);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let mappings = reader.rest(|reader| {
            Ok(ControlBusRange {
                control: Control::read(reader)?,
                bus: reader.int()?,
                count: reader.int()?,
            })
        })?;
        Ok(Self { node, mappings })
    }
}

/// The arguments of `/n_before` and `/n_after`: nodes move just before, or
/// just after, their targets, into the groups of the targets. scsynth skips
/// a pair whose node or target is missing.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NBefore {
    /// The moves, in order.
    pub moves: Vec<NodeMove>,
}

/// A move of `/n_before` or `/n_after`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeMove {
    /// The node that moves.
    pub node: NodeId,
    /// The node that it moves next to.
    pub target: NodeId,
}

/// The arguments of `/n_after`: see [`NBefore`].
pub type NAfter = NBefore;

impl Payload for NBefore {
    fn write(&self, writer: &mut Writer) {
        for entry in &self.moves {
            entry.node.write(writer);
            entry.target.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let moves = reader.rest(|reader| {
            Ok(NodeMove {
                node: NodeId::read(reader)?,
                target: NodeId::read(reader)?,
            })
        })?;
        Ok(Self { moves })
    }
}

/// The arguments of `/n_query`: scsynth sends `/n_info` for each node to
/// every registered client.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NQuery {
    /// The nodes.
    pub nodes: Vec<NodeId>,
}

impl Payload for NQuery {
    fn write(&self, writer: &mut Writer) {
        write_nodes(writer, &self.nodes);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            nodes: reader.rest(NodeId::read)?,
        })
    }
}

/// The arguments of `/n_trace`: for one control period, scsynth prints the
/// inputs and outputs of the unit generators of each synth, and the nodes
/// of each group.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NTrace {
    /// The nodes.
    pub nodes: Vec<NodeId>,
}

impl Payload for NTrace {
    fn write(&self, writer: &mut Writer) {
        write_nodes(writer, &self.nodes);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            nodes: reader.rest(NodeId::read)?,
        })
    }
}

/// The arguments of `/n_order`: nodes move next to a target, in the order
/// given. scsynth skips the nodes that are missing, and fails when none is
/// left.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NOrder {
    /// Where the nodes go, relative to the target.
    pub action: OrderAction,
    /// The target: a group for [`OrderAction::Head`] and
    /// [`OrderAction::Tail`], a node otherwise.
    pub target: NodeId,
    /// The nodes, in their new order.
    pub nodes: Vec<NodeId>,
}

impl Payload for NOrder {
    fn write(&self, writer: &mut Writer) {
        writer.int(self.action.code());
        self.target.write(writer);
        write_nodes(writer, &self.nodes);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            action: OrderAction::read(reader)?,
            target: NodeId::read(reader)?,
            nodes: reader.rest(NodeId::read)?,
        })
    }
}

/// Writes each node.
pub(crate) fn write_nodes(writer: &mut Writer, nodes: &[NodeId]) {
    for node in nodes {
        node.write(writer);
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};
    use crate::types::{BusMapping, ControlValue};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    fn nodes() -> Vec<NodeId> {
        vec![NodeId(1000), NodeId(1001)]
    }

    #[test]
    fn the_node_lists_have_their_wire_form() {
        let wire = || vec![Int(1000), Int(1001)];
        assert_wire(&Command::NFree(NFree { nodes: nodes() }), "/n_free", wire());
        assert_wire(
            &Command::NQuery(NQuery { nodes: nodes() }),
            "/n_query",
            wire(),
        );
        assert_wire(
            &Command::NTrace(NTrace { nodes: nodes() }),
            "/n_trace",
            wire(),
        );
        assert_wire(&Command::NFree(NFree { nodes: vec![] }), "/n_free", vec![]);
    }

    #[test]
    fn a_run_flag_is_1_or_0() {
        let run = NRun {
            nodes: vec![
                NodeRun {
                    node: NodeId(1000),
                    run: false,
                },
                NodeRun {
                    node: NodeId(1001),
                    run: true,
                },
            ],
        };
        let wire = vec![Int(1000), Int(0), Int(1001), Int(1)];
        assert_wire(&Command::NRun(run), "/n_run", wire);
        let kind = decode_error::<Command>("/n_run", vec![Int(1000), Int(2)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Command>("/n_run", vec![Int(1000)]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "an int"
            }
        );
    }

    #[test]
    fn a_set_takes_every_form_of_value() {
        let set = NSet {
            node: NodeId(1000),
            controls: vec![
                ControlSetting::new("freq", 440.0),
                ControlSetting::new(1, 2),
                ControlSetting::new("amp", BusMapping::Control(3)),
                ControlSetting::new("in", BusMapping::Audio(4)),
            ],
        };
        let wire = vec![
            Int(1000),
            text("freq"),
            Float(440.0),
            Int(1),
            Int(2),
            text("amp"),
            text("c3"),
            text("in"),
            text("a4"),
        ];
        assert_wire(&Command::NSet(set), "/n_set", wire);
    }

    #[test]
    fn a_set_needs_a_value_for_each_control() {
        assert_eq!(
            decode_error::<Command>("/n_set", vec![Int(1000), text("freq")]),
            DecodeErrorKind::Missing {
                index: 2,
                expected: "a control value: an int, a float, a bus mapping or an array"
            }
        );
    }

    #[test]
    fn a_range_counts_its_values() {
        let setn = NSetn {
            node: NodeId(1000),
            ranges: vec![
                ControlRange {
                    control: Control::from("freqs"),
                    values: vec![Number::Float(1.5), Number::Int(2)],
                },
                ControlRange {
                    control: Control::Index(4),
                    values: vec![],
                },
            ],
        };
        let wire = vec![
            Int(1000),
            text("freqs"),
            Int(2),
            Float(1.5),
            Int(2),
            Int(4),
            Int(0),
        ];
        assert_wire(&Command::NSetn(setn), "/n_setn", wire);
        let kind = decode_error::<Command>("/n_setn", vec![Int(1000), Int(0), Int(-1)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Command>("/n_setn", vec![Int(1000), Int(0), Int(2), Float(1.0)]),
            DecodeErrorKind::Missing {
                index: 4,
                expected: "an int or a float"
            }
        );
    }

    #[test]
    fn a_fill_writes_its_value_twice() {
        let fill = NFill {
            node: NodeId(1000),
            ranges: vec![
                ControlFill {
                    control: Control::from("freqs"),
                    count: 3,
                    value: Number::Float(0.5),
                },
                ControlFill {
                    control: Control::Index(8),
                    count: 2,
                    value: Number::Int(1),
                },
            ],
        };
        let wire = vec![
            Int(1000),
            text("freqs"),
            Int(3),
            Float(0.5),
            Float(0.5),
            Int(8),
            Int(2),
            Int(1),
            Int(1),
        ];
        assert_wire(&Command::NFill(fill), "/n_fill", wire);
        let nan = NFill {
            node: NodeId(1000),
            ranges: vec![ControlFill {
                control: Control::Index(0),
                count: 1,
                value: Number::Float(f32::NAN),
            }],
        };
        let bytes = Command::NFill(nan).encode();
        let Ok(Command::NFill(back)) = Command::decode(&bytes) else {
            panic!("a NaN fill decodes");
        };
        assert!(matches!(back.ranges[0].value, Number::Float(value) if value.is_nan()));
    }

    #[test]
    fn a_fill_of_the_reference_is_refused() {
        // scsynth 3.14.1 sets the controls to 0 for this message.
        let triple = vec![Int(1000), text("freq"), Int(3), Float(0.5)];
        assert_eq!(
            decode_error::<Command>("/n_fill", triple),
            DecodeErrorKind::Missing {
                index: 4,
                expected: "an int or a float"
            }
        );
        let differing = vec![Int(1000), text("freq"), Int(3), Float(0.5), Float(0.25)];
        let kind = decode_error::<Command>("/n_fill", differing);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 4, .. }),
            "{kind:?}"
        );
        let retagged = vec![Int(1000), text("freq"), Int(3), Int(1), Float(1.0)];
        let kind = decode_error::<Command>("/n_fill", retagged);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 4, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn the_maps_have_their_wire_form() {
        let map = NMap {
            node: NodeId(1000),
            mappings: vec![
                ControlBus {
                    control: Control::from("freq"),
                    bus: 3,
                },
                ControlBus {
                    control: Control::Index(1),
                    bus: -1,
                },
            ],
        };
        let wire = || vec![Int(1000), text("freq"), Int(3), Int(1), Int(-1)];
        assert_wire(&Command::NMap(map.clone()), "/n_map", wire());
        assert_wire(&Command::NMapa(map), "/n_mapa", wire());
        let mapn = NMapn {
            node: NodeId(1000),
            mappings: vec![ControlBusRange {
                control: Control::from("freqs"),
                bus: 8,
                count: 4,
            }],
        };
        let wire = || vec![Int(1000), text("freqs"), Int(8), Int(4)];
        assert_wire(&Command::NMapn(mapn.clone()), "/n_mapn", wire());
        assert_wire(&Command::NMapan(mapn), "/n_mapan", wire());
        assert_eq!(
            decode_error::<Command>("/n_mapn", vec![Int(1000), text("freqs"), Int(8)]),
            DecodeErrorKind::Missing {
                index: 3,
                expected: "an int"
            }
        );
    }

    #[test]
    fn a_map_takes_a_bus_index_not_a_bus_mapping() {
        assert_eq!(
            decode_error::<Command>("/n_map", vec![Int(1000), text("freq"), text("c3")]),
            DecodeErrorKind::WrongType {
                index: 2,
                expected: "an int",
                found: 's'
            }
        );
    }

    #[test]
    fn the_moves_have_their_wire_form() {
        let moves = NBefore {
            moves: vec![
                NodeMove {
                    node: NodeId(1001),
                    target: NodeId(1000),
                },
                NodeMove {
                    node: NodeId(1002),
                    target: NodeId(1001),
                },
            ],
        };
        let wire = || vec![Int(1001), Int(1000), Int(1002), Int(1001)];
        assert_wire(&Command::NBefore(moves.clone()), "/n_before", wire());
        assert_wire(&Command::NAfter(moves), "/n_after", wire());
        assert_eq!(
            decode_error::<Command>("/n_after", vec![Int(1001)]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "an int"
            }
        );
    }

    #[test]
    fn an_order_takes_four_actions() {
        let order = NOrder {
            action: OrderAction::After,
            target: NodeId(1000),
            nodes: nodes(),
        };
        let wire = vec![Int(3), Int(1000), Int(1000), Int(1001)];
        assert_wire(&Command::NOrder(order), "/n_order", wire);
        let kind = decode_error::<Command>("/n_order", vec![Int(4), Int(1000), Int(1001)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn a_control_is_an_index_or_a_name() {
        assert_eq!(
            decode_error::<Command>("/n_set", vec![Int(1000), Float(0.0), Float(1.0)]),
            DecodeErrorKind::WrongType {
                index: 1,
                expected: "a control index or name",
                found: 'f'
            }
        );
        let set = NSet {
            node: NodeId::RECENT,
            controls: vec![ControlSetting {
                control: Control::Index(0),
                value: ControlValue::Int(1),
            }],
        };
        assert_wire(&Command::NSet(set), "/n_set", vec![Int(-1), Int(0), Int(1)]);
    }
}
