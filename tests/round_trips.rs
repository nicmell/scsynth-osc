//! The values whose bytes do not decode back to them: see the README.

use scsynth_osc::command::buffer::BGen;
use scsynth_osc::command::buffer_fill::BufferFill;
use scsynth_osc::command::synth_def::DRecv;
use scsynth_osc::command::top_level::Cmd;
use scsynth_osc::reply::command_replies::{Done, Fail};
use scsynth_osc::reply::group::{GQueryTreeReply, TreeGroup, TreeNode};
use scsynth_osc::reply::node_notification::{NodeKind, NodeState};
use scsynth_osc::{Argument, BufferId, Command, NodeId, Reply};

#[test]
fn a_string_ends_at_its_first_nul_byte() {
    let fail = |message: &str| {
        Reply::Fail(Fail {
            command: "/s_new".to_owned(),
            message: message.to_owned(),
            value: None,
        })
    };
    assert_eq!(Reply::decode(&fail("a\0b").encode()), Ok(fail("a")));
    let cmd = |name: &str, arg: &str| {
        Command::Cmd(Cmd {
            name: name.to_owned(),
            args: vec![Argument::String(arg.to_owned())],
        })
    };
    assert_eq!(
        Command::decode(&cmd("demo\0x", "b\0c").encode()),
        Ok(cmd("demo", "b"))
    );
}

#[test]
fn a_synth_definition_without_bytes_does_not_decode() {
    let empty = Command::DRecv(DRecv {
        data: vec![],
        completion: None,
    });
    assert!(Command::decode(&empty.encode()).is_err());
}

#[test]
fn a_plugin_with_the_name_of_another_variant_does_not_decode_as_a_plugin() {
    let quit = Reply::Done(Done::Plugin {
        name: "/quit".to_owned(),
    });
    assert_eq!(Reply::decode(&quit.encode()), Ok(Reply::Done(Done::Quit)));
    let alloc = Reply::Done(Done::Plugin {
        name: "/b_alloc".to_owned(),
    });
    assert!(Reply::decode(&alloc.encode()).is_err());
    let sine = Command::BGen(BGen {
        buffer: BufferId::from(4),
        fill: BufferFill::Plugin {
            name: "sine1".to_owned(),
            args: vec![],
        },
    });
    assert!(Command::decode(&sine.encode()).is_err());
}

#[test]
fn a_neighbour_of_minus_1_decodes_as_none() {
    let state = NodeState {
        node: NodeId(1000),
        parent: Some(NodeId(0)),
        prev: Some(NodeId(-1)),
        next: None,
        kind: NodeKind::Synth,
    };
    let decoded = NodeState {
        prev: None,
        ..state
    };
    assert_eq!(
        Reply::decode(&Reply::NGo(state).encode()),
        Ok(Reply::NGo(decoded))
    );
}

#[test]
fn a_tree_of_129_groups_does_not_decode() {
    let mut tree = TreeGroup {
        id: NodeId(128),
        children: vec![],
    };
    for id in (0..128).rev() {
        tree = TreeGroup {
            id: NodeId(id),
            children: vec![TreeNode::Group(tree)],
        };
    }
    let reply = Reply::GQueryTreeReply(GQueryTreeReply::WithoutControls(tree));
    assert!(Reply::decode(&reply.encode()).is_err());
}
