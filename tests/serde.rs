//! The JSON form of the messages, with the feature `serde`.
#![cfg(feature = "serde")]

use scsynth_osc::command::buffer::{BGen, BWrite, HeaderFormat, SampleFormat};
use scsynth_osc::command::buffer_fill::{BufferFill, WaveFillFlags};
use scsynth_osc::command::control_bus::CSet;
use scsynth_osc::command::synth::SNew;
use scsynth_osc::command::top_level::{DumpMode, Notify, SyncId};
use scsynth_osc::reply::group::{
    GQueryTreeReply, TreeControl, TreeGroup, TreeNode, TreeSynth, TreeValue,
};
use scsynth_osc::reply::node_notification::{NodeKind, NodeState};
use scsynth_osc::reply::top_level::StatusReply;
use scsynth_osc::{
    AddAction, ArrayElement, BufferId, BusMapping, Command, Control, ControlSetting, ControlValue,
    IndexValue, NewNodeId, NodeId, Number, Reply,
};
use serde_json::json;

fn round_trip<T>(value: &T, json: serde_json::Value)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    assert_eq!(serde_json::to_value(value).expect("serializes"), json);
    let back: T = serde_json::from_value(json).expect("deserializes");
    assert_eq!(&back, value);
}

#[test]
fn a_command_is_its_address_and_its_args() {
    round_trip(&Command::Status, json!({"address": "/status"}));
    round_trip(
        &Command::Sync(SyncId(7)),
        json!({"address": "/sync", "args": 7}),
    );
    round_trip(
        &Command::DumpOsc(DumpMode::Parsed),
        json!({"address": "/dumpOSC", "args": "parsed"}),
    );
    let notify = Notify {
        on: true,
        client_id: None,
    };
    round_trip(
        &Command::Notify(notify),
        json!({"address": "/notify", "args": {"on": true, "client_id": null}}),
    );
}

#[test]
fn a_reply_is_its_address_and_its_fields() {
    let status = StatusReply {
        ugens: 1,
        synths: 2,
        groups: 3,
        synthdefs: 4,
        avg_cpu: 0.5,
        peak_cpu: 1.5,
        nominal_sample_rate: 48_000.0,
        actual_sample_rate: 47_999.5,
    };
    round_trip(
        &Reply::StatusReply(status),
        json!({"address": "/status.reply", "args": {
            "ugens": 1, "synths": 2, "groups": 3, "synthdefs": 4,
            "avg_cpu": 0.5, "peak_cpu": 1.5,
            "nominal_sample_rate": 48000.0, "actual_sample_rate": 47999.5
        }}),
    );
}

#[test]
fn a_synth_names_its_controls_and_their_values() {
    let new = SNew {
        def: "default".to_owned(),
        id: NewNodeId::AUTO,
        action: AddAction::Head,
        target: NodeId(1),
        controls: vec![
            ControlSetting::new("freq", 440.0),
            ControlSetting::new(0, BusMapping::Control(3)),
            ControlSetting::new(
                "partials",
                ControlValue::Array(vec![
                    ArrayElement::Float(0.5),
                    ArrayElement::Int(1),
                    ArrayElement::Bus(BusMapping::Audio(2)),
                ]),
            ),
        ],
    };
    round_trip(
        &Command::SNew(new),
        json!({"address": "/s_new", "args": {
            "def": "default", "id": -1, "action": "head", "target": 1,
            "controls": [
                {"control": "freq", "value": 440.0},
                {"control": 0, "value": "c3"},
                {"control": "partials", "value": [0.5, 1, "a2"]}
            ]
        }}),
    );
    let refused = json!({"address": "/s_new", "args": {
        "def": "default", "id": -2, "action": "head", "target": 1, "controls": []
    }});
    assert!(serde_json::from_value::<Command>(refused).is_err());
}

#[test]
fn a_notification_names_the_neighbours_of_its_node() {
    let state = NodeState {
        node: NodeId(1000),
        parent: Some(NodeId(1)),
        prev: None,
        next: Some(NodeId(1001)),
        kind: NodeKind::Synth,
    };
    round_trip(
        &Reply::NGo(state),
        json!({"address": "/n_go", "args": {
            "node": 1000, "parent": 1, "prev": null, "next": 1001, "kind": "synth"
        }}),
    );
    let group = NodeState {
        kind: NodeKind::Group {
            head: None,
            tail: Some(NodeId(1002)),
        },
        ..state
    };
    round_trip(
        &Reply::NEnd(group),
        json!({"address": "/n_end", "args": {
            "node": 1000, "parent": 1, "prev": null, "next": 1001,
            "kind": {"group": {"head": null, "tail": 1002}}
        }}),
    );
}

#[test]
fn a_tree_nests_its_groups() {
    let tree = TreeGroup {
        id: NodeId(1),
        children: vec![
            TreeNode::Group(TreeGroup {
                id: NodeId(2),
                children: vec![],
            }),
            TreeNode::Synth(TreeSynth {
                id: NodeId(1000),
                def: "default".to_owned(),
                controls: vec![TreeControl {
                    control: Control::from("freq"),
                    value: TreeValue::Bus(BusMapping::Control(3)),
                }],
            }),
        ],
    };
    round_trip(
        &Reply::GQueryTreeReply(GQueryTreeReply::WithControls(tree)),
        json!({"address": "/g_queryTree.reply", "args": {"with_controls": {
            "id": 1,
            "children": [
                {"group": {"id": 2, "children": []}},
                {"synth": {"id": 1000, "def": "default", "controls": [
                    {"control": "freq", "value": "c3"}
                ]}}
            ]
        }}}),
    );
}

#[test]
fn a_buffer_command_names_its_buffer_and_formats() {
    let write = BWrite {
        buffer: BufferId::from(4),
        path: "a.wav".to_owned(),
        header: HeaderFormat::Wav,
        sample: SampleFormat::Int24,
        frames: -1,
        start_frame: 0,
        leave_open: false,
        completion: None,
    };
    round_trip(
        &Command::BWrite(write),
        json!({"address": "/b_write", "args": {
            "buffer": 4, "path": "a.wav", "header": "wav", "sample": "int24",
            "frames": -1, "start_frame": 0, "leave_open": false, "completion": null
        }}),
    );
    let fill = BGen {
        buffer: BufferId::from(4),
        fill: BufferFill::Sine1 {
            flags: WaveFillFlags {
                normalize: true,
                wavetable: false,
                clear: true,
            },
            amplitudes: vec![1.0, 0.5],
        },
    };
    round_trip(
        &Command::BGen(fill),
        json!({"address": "/b_gen", "args": {"buffer": 4, "fill": {"sine1": {
            "flags": {"normalize": true, "wavetable": false, "clear": true},
            "amplitudes": [1.0, 0.5]
        }}}}),
    );
    let negative = json!({"address": "/b_free", "args": {"buffer": -1, "completion": null}});
    assert!(serde_json::from_value::<Command>(negative).is_err());
    let channels: Vec<u16> = (0..33).collect();
    let read = json!({"address": "/b_allocReadChannel", "args": {
        "buffer": 4, "path": "a.wav", "start_frame": 0, "frames": -1,
        "channels": channels, "completion": null
    }});
    assert!(serde_json::from_value::<Command>(read).is_err());
}

#[test]
fn a_bus_value_is_an_int_or_a_float() {
    let set = CSet {
        buses: vec![
            IndexValue {
                index: 0,
                value: Number::Int(1),
            },
            IndexValue {
                index: 1,
                value: Number::Float(0.5),
            },
        ],
    };
    round_trip(
        &Command::CSet(set),
        json!({"address": "/c_set", "args": {"buses": [
            {"index": 0, "value": 1},
            {"index": 1, "value": 0.5}
        ]}}),
    );
}
