//! Node notifications: scsynth sends them to every registered client when a
//! node starts, ends, turns off or on, or moves, and for `/n_query`. They
//! share their arguments, a [`NodeState`].
//!
//! scsynth sends none for a node whose ID it picked, except `/n_info`.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::NodeId;

/// The arguments of `/n_go`, `/n_end`, `/n_off`, `/n_on`, `/n_move` and
/// `/n_info`: a node, and its place in the tree.
///
/// A missing node travels as -1, which no node has, so `Some(NodeId(-1))`
/// encodes as `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeState {
    /// The node.
    pub node: NodeId,
    /// The group of the node: none for the root group.
    pub parent: Option<NodeId>,
    /// The node before it in its group.
    pub prev: Option<NodeId>,
    /// The node after it in its group.
    pub next: Option<NodeId>,
    /// Whether the node is a synth or a group.
    pub kind: NodeKind,
}

/// A synth, or a group with its first and last nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum NodeKind {
    /// A synth.
    Synth,
    /// A group.
    Group {
        /// The node at its head, which runs first.
        head: Option<NodeId>,
        /// The node at its tail, which runs last.
        tail: Option<NodeId>,
    },
}

impl Payload for NodeState {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        write_link(writer, self.parent);
        write_link(writer, self.prev);
        write_link(writer, self.next);
        match self.kind {
            NodeKind::Synth => writer.int(0),
            NodeKind::Group { head, tail } => {
                writer.int(1);
                write_link(writer, head);
                write_link(writer, tail);
            }
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let parent = read_link(reader)?;
        let prev = read_link(reader)?;
        let next = read_link(reader)?;
        let kind = if reader.flag()? {
            NodeKind::Group {
                head: read_link(reader)?,
                tail: read_link(reader)?,
            }
        } else {
            NodeKind::Synth
        };
        Ok(Self {
            node,
            parent,
            prev,
            next,
            kind,
        })
    }
}

/// A node that may be missing: -1 on the wire, which no node has.
fn read_link(reader: &mut Reader<'_>) -> Result<Option<NodeId>, DecodeError> {
    let node = NodeId::read(reader)?;
    Ok((node != NodeId(-1)).then_some(node))
}

fn write_link(writer: &mut Writer, node: Option<NodeId>) {
    node.unwrap_or(NodeId(-1)).write(writer);
}

#[cfg(test)]
mod tests {
    use rosc::OscType::Int;

    use super::*;
    use crate::Reply;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    fn synth() -> NodeState {
        NodeState {
            node: NodeId(1001),
            parent: Some(NodeId(1)),
            prev: Some(NodeId(1000)),
            next: None,
            kind: NodeKind::Synth,
        }
    }

    #[test]
    fn a_synth_has_five_arguments_and_a_group_seven() {
        let wire = || vec![Int(1001), Int(1), Int(1000), Int(-1), Int(0)];
        for (reply, address) in [
            (Reply::NGo(synth()), "/n_go"),
            (Reply::NEnd(synth()), "/n_end"),
            (Reply::NOff(synth()), "/n_off"),
            (Reply::NOn(synth()), "/n_on"),
            (Reply::NMove(synth()), "/n_move"),
            (Reply::NInfo(synth()), "/n_info"),
        ] {
            assert_wire(&reply, address, wire());
        }
        let root = NodeState {
            node: NodeId::ROOT,
            parent: None,
            prev: None,
            next: None,
            kind: NodeKind::Group {
                head: Some(NodeId(1)),
                tail: Some(NodeId(2)),
            },
        };
        let wire = vec![Int(0), Int(-1), Int(-1), Int(-1), Int(1), Int(1), Int(2)];
        assert_wire(&Reply::NInfo(root), "/n_info", wire);
    }

    #[test]
    fn the_kind_decides_the_arity() {
        assert_eq!(
            decode_error::<Reply>("/n_go", vec![Int(1), Int(0), Int(-1), Int(-1), Int(1)]),
            DecodeErrorKind::Missing {
                index: 5,
                expected: "an int"
            }
        );
        assert_eq!(
            decode_error::<Reply>(
                "/n_go",
                vec![
                    Int(1001),
                    Int(1),
                    Int(-1),
                    Int(-1),
                    Int(0),
                    Int(-1),
                    Int(-1)
                ]
            ),
            DecodeErrorKind::Trailing { index: 5 }
        );
        let kind =
            decode_error::<Reply>("/n_go", vec![Int(1001), Int(1), Int(-1), Int(-1), Int(2)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 4, .. }),
            "{kind:?}"
        );
    }
}
