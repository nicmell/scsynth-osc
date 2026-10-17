//! Messages byte for byte as scsynth 3.14.1 sent them, decoded strictly.

use rosc::OscType::{Int, String as Text};
use rosc::{OscMessage, OscPacket, OscType};
use scsynth_osc::Reply;
use scsynth_osc::reply::command_replies::{Done, Fail};

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
        max_logins: 4,
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
