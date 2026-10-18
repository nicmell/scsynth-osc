//! Group commands.

use crate::codec::{Payload, Reader, Writer};
use crate::error::DecodeError;
use crate::types::{AddAction, NewNodeId, NodeId};

use super::node::write_nodes;

/// The arguments of `/g_new` and `/p_new`: groups, each placed relative to
/// a target. scsynth sends `/n_go` to registered clients for each group,
/// except for an ID that it picked. An ID in use fails, with an exception:
/// see [Differences](crate#differences-from-the-reference).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GNew {
    /// The groups, created in order.
    pub groups: Vec<NewGroup>,
}

/// A group of `/g_new` or `/p_new`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewGroup {
    /// The ID of the group.
    pub id: NewNodeId,
    /// Where the group goes, relative to the target.
    pub action: AddAction,
    /// The target: a group for [`AddAction::Head`] and [`AddAction::Tail`],
    /// a node otherwise. The root group cannot be replaced.
    pub target: NodeId,
}

/// The arguments of `/p_new`: see [`GNew`].
pub type PNew = GNew;

impl Payload for GNew {
    fn write(&self, writer: &mut Writer) {
        for group in &self.groups {
            group.id.write(writer);
            writer.int(group.action.code());
            group.target.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let groups = reader.rest(|reader| {
            Ok(NewGroup {
                id: NewNodeId::read(reader)?,
                action: AddAction::read(reader)?,
                target: NodeId::read(reader)?,
            })
        })?;
        Ok(Self { groups })
    }
}

/// The arguments of `/g_head` and `/g_tail`: nodes move to the head, or to
/// the tail, of groups. The head runs first.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GHead {
    /// The moves, in order.
    pub moves: Vec<GroupMove>,
}

/// A move of `/g_head` or `/g_tail`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupMove {
    /// The group.
    pub group: NodeId,
    /// The node that moves into it.
    pub node: NodeId,
}

/// The arguments of `/g_tail`: see [`GHead`].
pub type GTail = GHead;

impl Payload for GHead {
    fn write(&self, writer: &mut Writer) {
        for entry in &self.moves {
            entry.group.write(writer);
            entry.node.write(writer);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let moves = reader.rest(|reader| {
            Ok(GroupMove {
                group: NodeId::read(reader)?,
                node: NodeId::read(reader)?,
            })
        })?;
        Ok(Self { moves })
    }
}

/// The arguments of `/g_freeAll` and `/g_deepFree`. `/g_freeAll` frees
/// every node in the groups. `/g_deepFree` frees every synth in the groups
/// and in the groups inside them, and keeps the groups.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GFreeAll {
    /// The groups.
    pub groups: Vec<NodeId>,
}

/// The arguments of `/g_deepFree`: see [`GFreeAll`].
pub type GDeepFree = GFreeAll;

impl Payload for GFreeAll {
    fn write(&self, writer: &mut Writer) {
        write_nodes(writer, &self.groups);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            groups: reader.rest(NodeId::read)?,
        })
    }
}

/// The arguments of `/g_dumpTree` and `/g_queryTree`. `/g_dumpTree` makes
/// scsynth print the tree of each group. `/g_queryTree` makes it reply
/// `/g_queryTree.reply` for each group: see
/// [`crate::reply::group::GQueryTreeReply`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GDumpTree {
    /// The groups.
    pub groups: Vec<TreeRequest>,
}

/// A group of `/g_dumpTree` or `/g_queryTree`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreeRequest {
    /// The group.
    pub group: NodeId,
    /// Whether the tree holds the current values of the controls of the
    /// synths.
    pub controls: bool,
}

/// The arguments of `/g_queryTree`: see [`GDumpTree`].
pub type GQueryTree = GDumpTree;

impl Payload for GDumpTree {
    fn write(&self, writer: &mut Writer) {
        for request in &self.groups {
            request.group.write(writer);
            writer.flag(request.controls);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let groups = reader.rest(|reader| {
            Ok(TreeRequest {
                group: NodeId::read(reader)?,
                controls: reader.flag()?,
            })
        })?;
        Ok(Self { groups })
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::Int;

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn new_groups_are_triples() {
        let groups = GNew {
            groups: vec![
                NewGroup {
                    id: NewNodeId::new(1000).expect("an ID of 0 or more"),
                    action: AddAction::Tail,
                    target: NodeId::ROOT,
                },
                NewGroup {
                    id: NewNodeId::AUTO,
                    action: AddAction::Before,
                    target: NodeId(1000),
                },
            ],
        };
        let wire = || vec![Int(1000), Int(1), Int(0), Int(-1), Int(2), Int(1000)];
        assert_wire(&Command::GNew(groups.clone()), "/g_new", wire());
        assert_wire(&Command::PNew(groups), "/p_new", wire());
        assert_eq!(
            decode_error::<Command>("/g_new", vec![Int(1000), Int(1)]),
            DecodeErrorKind::Missing {
                index: 2,
                expected: "an int"
            }
        );
        let kind = decode_error::<Command>("/g_new", vec![Int(-5), Int(1), Int(0)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
            "{kind:?}"
        );
    }

    #[test]
    fn the_moves_and_frees_have_their_wire_form() {
        let moves = GHead {
            moves: vec![GroupMove {
                group: NodeId(1),
                node: NodeId(1000),
            }],
        };
        assert_wire(
            &Command::GHead(moves.clone()),
            "/g_head",
            vec![Int(1), Int(1000)],
        );
        assert_wire(&Command::GTail(moves), "/g_tail", vec![Int(1), Int(1000)]);
        let groups = GFreeAll {
            groups: vec![NodeId(1), NodeId(2)],
        };
        assert_wire(
            &Command::GFreeAll(groups.clone()),
            "/g_freeAll",
            vec![Int(1), Int(2)],
        );
        assert_wire(
            &Command::GDeepFree(groups),
            "/g_deepFree",
            vec![Int(1), Int(2)],
        );
    }

    #[test]
    fn a_tree_request_flags_the_controls() {
        let request = GDumpTree {
            groups: vec![
                TreeRequest {
                    group: NodeId::ROOT,
                    controls: true,
                },
                TreeRequest {
                    group: NodeId(1),
                    controls: false,
                },
            ],
        };
        let wire = || vec![Int(0), Int(1), Int(1), Int(0)];
        assert_wire(&Command::GDumpTree(request.clone()), "/g_dumpTree", wire());
        assert_wire(&Command::GQueryTree(request), "/g_queryTree", wire());
        let kind = decode_error::<Command>("/g_queryTree", vec![Int(0), Int(2)]);
        assert!(
            matches!(kind, DecodeErrorKind::Invalid { index: 1, .. }),
            "{kind:?}"
        );
        assert_eq!(
            decode_error::<Command>("/g_queryTree", vec![Int(0)]),
            DecodeErrorKind::Missing {
                index: 1,
                expected: "an int"
            }
        );
    }
}
