//! Messages byte for byte as scsynth 3.14.1 sent them, decoded strictly.

use rosc::OscType::{Int, String as Text};
use rosc::{OscMessage, OscPacket, OscType};
use scsynth_osc::reply::command_replies::{Done, Fail};
use scsynth_osc::reply::group::{GQueryTreeReply, TreeGroup, TreeNode};
use scsynth_osc::reply::node_notification::{NodeKind, NodeState};
use scsynth_osc::{NodeId, Reply};

fn bytes(address: &str, args: Vec<OscType>) -> Vec<u8> {
    rosc::encoder::encode(&OscPacket::Message(OscMessage {
        addr: address.to_owned(),
        args,
    }))
    .expect("an OSC message encodes")
}

/// Verbatim from scsynth 3.14.1 with `-l 4`, to its first client.
#[test]
fn a_registration_names_the_client_and_the_login_limit() {
    let reply = bytes("/done", vec![Text("/notify".to_owned()), Int(0), Int(4)]);
    let done = Done::Notify {
        client_id: 0,
        max_logins: Some(4),
    };
    assert_eq!(Reply::decode(&reply), Ok(Reply::Done(done)));
}

/// Verbatim from scsynth 3.14.1 with `-l 1`, to a second client.
#[test]
fn a_full_server_refuses_with_a_trailing_newline() {
    let reply = bytes(
        "/fail",
        vec![
            Text("/notify".to_owned()),
            Text("too many users\n".to_owned()),
        ],
    );
    let fail = Fail {
        command: "/notify".to_owned(),
        message: "too many users\n".to_owned(),
        value: None,
    };
    assert_eq!(Reply::decode(&reply), Ok(Reply::Fail(fail)));
}

/// Verbatim from scsynth 3.14.1: the failure of a query for a group that
/// does not exist.
#[test]
fn a_failure_names_its_command() {
    let reply = bytes(
        "/fail",
        vec![
            Text("/g_queryTree".to_owned()),
            Text("Group 4242 not found".to_owned()),
        ],
    );
    let fail = Fail {
        command: "/g_queryTree".to_owned(),
        message: "Group 4242 not found".to_owned(),
        value: None,
    };
    assert_eq!(Reply::decode(&reply), Ok(Reply::Fail(fail)));
}

/// Verbatim from scsynth 3.14.1: a group 7001 in group 7000.
#[test]
fn a_node_notification_places_its_node() {
    let state = NodeState {
        node: NodeId(7001),
        parent: Some(NodeId(7000)),
        prev: None,
        next: None,
        kind: NodeKind::Group {
            head: None,
            tail: None,
        },
    };
    for (address, reply) in [
        ("/n_go", Reply::NGo(state)),
        ("/n_end", Reply::NEnd(state)),
        ("/n_info", Reply::NInfo(state)),
    ] {
        let args = [7001, 7000, -1, -1, 1, -1, -1].map(Int).to_vec();
        assert_eq!(Reply::decode(&bytes(address, args)), Ok(reply));
    }
}

/// Verbatim from scsynth 3.14.1: an empty group, and a group with an empty
/// group in it.
#[test]
fn a_tree_holds_its_groups() {
    let empty = |id| TreeGroup {
        id: NodeId(id),
        children: vec![],
    };
    let nested = TreeGroup {
        id: NodeId(1000),
        children: vec![TreeNode::Group(empty(1001))],
    };
    for (args, tree) in [
        (vec![0, 2000, 0], empty(2000)),
        (vec![0, 1000, 1, 1001, 0], nested),
    ] {
        let reply = bytes("/g_queryTree.reply", args.into_iter().map(Int).collect());
        assert_eq!(
            Reply::decode(&reply),
            Ok(Reply::GQueryTreeReply(GQueryTreeReply::WithoutControls(
                tree
            )))
        );
    }
}
