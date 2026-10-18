//! The reply of the group commands: `/g_queryTree.reply`.

use rosc::OscType;

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{BusMapping, Control, NodeId};

/// The deepest tree that decoding takes, in groups from the root.
const MAX_DEPTH: usize = 128;

/// The arguments of `/g_queryTree.reply`: a group, and every node inside
/// it, in the order they run. The query chooses whether the synths carry
/// the values of their controls.
/// See [Limits](crate#limits) for its depth.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum GQueryTreeReply {
    /// The tree, without the controls of the synths.
    WithoutControls(TreeGroup<()>),
    /// The tree, with the controls of the synths.
    WithControls(TreeGroup<Vec<TreeControl>>),
}

/// A group of the tree, with the nodes inside it. `C` is what each synth
/// carries: `()`, or its controls.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreeGroup<C> {
    /// The group.
    pub id: NodeId,
    /// The nodes in the group, in the order they run.
    pub children: Vec<TreeNode<C>>,
}

/// A node of the tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum TreeNode<C> {
    /// A group.
    Group(TreeGroup<C>),
    /// A synth.
    Synth(TreeSynth<C>),
}

/// A synth of the tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreeSynth<C> {
    /// The synth.
    pub id: NodeId,
    /// The name of its synth definition.
    pub def: String,
    /// `()`, or its controls in the order of their indices.
    pub controls: C,
}

/// A control of a synth in the tree, and its value. A control without a
/// name has its index, such as the controls of an array after its first.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreeControl {
    /// The control.
    pub control: Control,
    /// Its value, or the bus that it reads.
    pub value: TreeValue,
}

/// The value of a control in the tree.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum TreeValue {
    /// The value.
    Float(f32),
    /// The bus that the control reads.
    Bus(BusMapping),
}

/// What a synth of the tree carries on the wire.
trait SynthControls: Sized {
    fn write(&self, writer: &mut Writer);

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError>;
}

impl SynthControls for () {
    fn write(&self, _writer: &mut Writer) {}

    fn read(_reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(())
    }
}

impl SynthControls for Vec<TreeControl> {
    fn write(&self, writer: &mut Writer) {
        writer.count(self.len());
        for entry in self {
            entry.control.write(writer);
            match entry.value {
                TreeValue::Float(value) => writer.float(value),
                TreeValue::Bus(mapping) => writer.string(&mapping.to_string()),
            }
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        const EXPECTED: &str = "a float or a bus mapping";
        let count = reader.count()?;
        (0..count)
            .map(|_| {
                let control = Control::read(reader)?;
                let index = reader.index();
                let value = match reader.any(EXPECTED)? {
                    OscType::Float(value) => TreeValue::Float(*value),
                    OscType::String(text) => {
                        TreeValue::Bus(BusMapping::parse_at(reader, index, text)?)
                    }
                    other => return Err(reader.wrong(other, EXPECTED)),
                };
                Ok(TreeControl { control, value })
            })
            .collect()
    }
}

fn write_group<C: SynthControls>(group: &TreeGroup<C>, writer: &mut Writer) {
    group.id.write(writer);
    writer.count(group.children.len());
    for child in &group.children {
        match child {
            TreeNode::Group(group) => write_group(group, writer),
            TreeNode::Synth(synth) => {
                synth.id.write(writer);
                writer.int(-1);
                writer.string(&synth.def);
                synth.controls.write(writer);
            }
        }
    }
}

/// The root of the tree: a group.
fn read_root<C: SynthControls>(reader: &mut Reader<'_>) -> Result<TreeGroup<C>, DecodeError> {
    let id = NodeId::read(reader)?;
    let count = reader.count()?;
    read_group(reader, id, count, 1)
}

/// The group `id` and its `count` children, `depth` groups from the root.
fn read_group<C: SynthControls>(
    reader: &mut Reader<'_>,
    id: NodeId,
    count: usize,
    depth: usize,
) -> Result<TreeGroup<C>, DecodeError> {
    let mut children = Vec::new();
    for _ in 0..count {
        let index = reader.index();
        let child = NodeId::read(reader)?;
        let count_index = reader.index();
        let count = reader.int()?;
        if count == -1 {
            children.push(TreeNode::Synth(TreeSynth {
                id: child,
                def: reader.string()?,
                controls: C::read(reader)?,
            }));
            continue;
        }
        let count = usize::try_from(count).map_err(|_| {
            reader.invalid(
                count_index,
                format!("{count} is no count of children: -1 for a synth, or 0 or more"),
            )
        })?;
        if depth == MAX_DEPTH {
            return Err(reader.invalid(
                index,
                format!("a group deeper than {MAX_DEPTH} groups, which the crate refuses"),
            ));
        }
        children.push(TreeNode::Group(read_group(
            reader,
            child,
            count,
            depth + 1,
        )?));
    }
    Ok(TreeGroup { id, children })
}

impl Payload for GQueryTreeReply {
    fn write(&self, writer: &mut Writer) {
        match self {
            Self::WithoutControls(group) => {
                writer.flag(false);
                write_group(group, writer);
            }
            Self::WithControls(group) => {
                writer.flag(true);
                write_group(group, writer);
            }
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        if reader.flag()? {
            read_root(reader).map(Self::WithControls)
        } else {
            read_root(reader).map(Self::WithoutControls)
        }
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Float, Int, String as Text};

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn a_tree_lists_its_nodes_in_the_order_they_run() {
        let tree = TreeGroup {
            id: NodeId::ROOT,
            children: vec![
                TreeNode::Group(TreeGroup {
                    id: NodeId(1),
                    children: vec![TreeNode::Synth(TreeSynth {
                        id: NodeId(1000),
                        def: "default".to_owned(),
                        controls: (),
                    })],
                }),
                TreeNode::Group(TreeGroup {
                    id: NodeId(2),
                    children: vec![],
                }),
            ],
        };
        let wire = vec![
            Int(0),
            Int(0),
            Int(2),
            Int(1),
            Int(1),
            Int(1000),
            Int(-1),
            text("default"),
            Int(2),
            Int(0),
        ];
        assert_wire(
            &Reply::GQueryTreeReply(GQueryTreeReply::WithoutControls(tree)),
            "/g_queryTree.reply",
            wire,
        );
    }

    #[test]
    fn a_tree_with_controls_names_their_values_or_buses() {
        let tree = TreeGroup {
            id: NodeId(1),
            children: vec![TreeNode::Synth(TreeSynth {
                id: NodeId(1000),
                def: "default".to_owned(),
                controls: vec![
                    TreeControl {
                        control: Control::from("freq"),
                        value: TreeValue::Float(440.0),
                    },
                    TreeControl {
                        control: Control::Index(1),
                        value: TreeValue::Bus(BusMapping::Audio(4)),
                    },
                ],
            })],
        };
        let wire = vec![
            Int(1),
            Int(1),
            Int(1),
            Int(1000),
            Int(-1),
            text("default"),
            Int(2),
            text("freq"),
            Float(440.0),
            Int(1),
            text("a4"),
        ];
        assert_wire(
            &Reply::GQueryTreeReply(GQueryTreeReply::WithControls(tree)),
            "/g_queryTree.reply",
            wire,
        );
    }

    #[test]
    fn a_tree_is_refused_where_its_counts_do_not_hold() {
        let kind = decode_error::<Reply>("/g_queryTree.reply", vec![Int(0), Int(1000), Int(-1)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 2, .. }),
            "a synth at the root: {kind:?}"
        );
        let kind = decode_error::<Reply>(
            "/g_queryTree.reply",
            vec![Int(0), Int(0), Int(1), Int(1000), Int(-2)],
        );
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 4, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Reply>(
                "/g_queryTree.reply",
                vec![Int(0), Int(0), Int(2), Int(1), Int(0)]
            ),
            DecodeErrorKind::Missing {
                index: 5,
                expected: "an int"
            }
        );
        assert_eq!(
            decode_error::<Reply>(
                "/g_queryTree.reply",
                vec![
                    Int(0),
                    Int(0),
                    Int(1),
                    Int(1000),
                    Int(-1),
                    text("default"),
                    Int(0)
                ]
            ),
            DecodeErrorKind::Trailing { index: 6 },
            "controls without the flag"
        );
        assert_eq!(
            decode_error::<Reply>(
                "/g_queryTree.reply",
                vec![Int(1), Int(0), Int(1), Int(1000), Int(-1), text("default")]
            ),
            DecodeErrorKind::Missing {
                index: 6,
                expected: "an int"
            },
            "the flag without controls"
        );
        assert_eq!(
            decode_error::<Reply>(
                "/g_queryTree.reply",
                vec![
                    Int(1),
                    Int(0),
                    Int(1),
                    Int(1000),
                    Int(-1),
                    text("default"),
                    Int(1),
                    text("freq"),
                    Int(440)
                ]
            ),
            DecodeErrorKind::WrongType {
                index: 8,
                expected: "a float or a bus mapping",
                found: 'i'
            }
        );
    }

    /// A chain of `depth` groups: the root and the groups inside it, one in
    /// each.
    fn chain(depth: usize) -> Vec<rosc::OscType> {
        let mut wire = vec![Int(0)];
        for id in 0..depth {
            let id = i32::try_from(id).expect("a small depth");
            wire.extend([Int(id), Int(1)]);
        }
        let last = wire.len() - 1;
        wire[last] = Int(0);
        wire
    }

    #[test]
    fn a_tree_is_refused_past_its_depth_limit() {
        let deepest = rosc::OscMessage {
            addr: "/g_queryTree.reply".to_owned(),
            args: chain(MAX_DEPTH),
        };
        assert!(<Reply as crate::codec::sealed::Codec>::from_osc(&deepest).is_ok());
        let kind = decode_error::<Reply>("/g_queryTree.reply", chain(MAX_DEPTH + 1));
        assert!(matches!(kind, DecodeErrorKind::Invalid { .. }), "{kind:?}");
    }
}
