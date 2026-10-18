//! The JSON form of the messages, with the feature `serde`.
#![cfg(feature = "serde")]

use scsynth_osc::command::buffer::{BFree, BGen, BWrite, HeaderFormat, SampleFormat};
use scsynth_osc::command::buffer_fill::{BufferFill, Partial, PhasedPartial, WaveFillFlags};
use scsynth_osc::command::control_bus::CSet;
use scsynth_osc::command::node::NOrder;
use scsynth_osc::command::pump::{Plugin, Root};
use scsynth_osc::command::synth::SNew;
use scsynth_osc::command::synth_def::DLoad;
use scsynth_osc::command::top_level::{Cmd, DumpMode, ErrorMode, Notify, SyncId};
use scsynth_osc::reply::command_replies::{BufferCommand, Done, Fail};
use scsynth_osc::reply::group::{
    GQueryTreeReply, TreeControl, TreeGroup, TreeNode, TreeSynth, TreeValue,
};
use scsynth_osc::reply::node_notification::{NodeKind, NodeState};
use scsynth_osc::reply::pump::PluginFailure;
use scsynth_osc::reply::top_level::StatusReply;
use scsynth_osc::reply::trigger::Tr;
use scsynth_osc::{
    AddAction, Argument, ArrayElement, BufferId, Bundle, BusMapping, Command, Completion, Control,
    ControlSetting, ControlValue, IndexValue, NewNodeId, NodeId, Number, OrderAction, Packet,
    RawArg, RawMessage, Reply, Timetag,
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
        buffer_frame: 0,
        leave_open: false,
        completion: None,
    };
    round_trip(
        &Command::BWrite(write),
        json!({"address": "/b_write", "args": {
            "buffer": 4, "path": "a.wav", "header": "wav", "sample": "int24",
            "frames": -1, "buffer_frame": 0, "leave_open": false, "completion": null
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
    let error = serde_json::from_value::<Command>(negative).unwrap_err();
    assert!(
        error.to_string().contains("-1 is not a buffer number"),
        "{error}"
    );
    let channels: Vec<u16> = (0..33).collect();
    let read = json!({"address": "/b_allocReadChannel", "args": {
        "buffer": 4, "path": "a.wav", "file_frame": 0, "frames": -1,
        "channels": channels, "completion": null
    }});
    let error = serde_json::from_value::<Command>(read).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("33 channels, where a read takes at most 32"),
        "{error}"
    );
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

#[test]
fn a_packet_is_a_message_or_a_bundle() {
    round_trip(
        &Packet::Message(Command::Status),
        json!({"message": {"address": "/status"}}),
    );
    let bundle = Packet::Bundle(Bundle {
        time: Timetag {
            seconds: 7,
            fraction: 9,
        },
        packets: vec![Packet::Message(Command::Sync(SyncId(1)))],
    });
    round_trip(
        &bundle,
        json!({"bundle": {
            "time": {"seconds": 7, "fraction": 9},
            "packets": [{"message": {"address": "/sync", "args": 1}}]
        }}),
    );
}

#[test]
fn a_completion_is_a_message_or_a_bundle() {
    let load = |completion| {
        Command::DLoad(DLoad {
            path: "a".to_owned(),
            completion: Some(completion),
        })
    };
    round_trip(
        &load(Completion::Message(Box::new(Command::Status))),
        json!({"address": "/d_load", "args": {
            "path": "a", "completion": {"message": {"address": "/status"}}
        }}),
    );
    let bundle = Completion::Bundle {
        time: Timetag::IMMEDIATELY,
        commands: vec![Command::Status],
    };
    round_trip(
        &load(bundle),
        json!({"address": "/d_load", "args": {"path": "a", "completion": {"bundle": {
            "time": {"seconds": 0, "fraction": 1}, "commands": [{"address": "/status"}]
        }}}}),
    );
}

#[test]
fn a_plugin_argument_names_its_type() {
    let cmd = Cmd {
        name: "demo".to_owned(),
        args: vec![
            Argument::Int(1),
            Argument::Float(0.5),
            Argument::Double(0.25),
            Argument::String("x".to_owned()),
            Argument::Blob(vec![1, 2]),
        ],
    };
    round_trip(
        &Command::Cmd(cmd),
        json!({"address": "/cmd", "args": {"name": "demo", "args": [
            {"int": 1}, {"float": 0.5}, {"double": 0.25}, {"string": "x"}, {"blob": [1, 2]}
        ]}}),
    );
}

#[test]
fn every_done_names_its_command() {
    let buffer = Done::Buffer {
        command: BufferCommand::BAllocReadChannel,
        buffer: BufferId::from(4),
    };
    let notify = Done::Notify {
        client_id: 0,
        max_logins: Some(32),
    };
    let plugin = Done::Plugin {
        name: "cmdDemo".to_owned(),
    };
    for (done, args) in [
        (Done::Quit, json!("quit")),
        (
            notify,
            json!({"notify": {"client_id": 0, "max_logins": 32}}),
        ),
        (Done::NotifyOff, json!("notify_off")),
        (Done::DRecv, json!("d_recv")),
        (Done::DLoad, json!("d_load")),
        (Done::DLoadDir, json!("d_load_dir")),
        (
            buffer,
            json!({"buffer": {"command": "/b_allocReadChannel", "buffer": 4}}),
        ),
        (plugin, json!({"plugin": {"name": "cmdDemo"}})),
    ] {
        round_trip(
            &Reply::Done(done),
            json!({"address": "/done", "args": args}),
        );
    }
    let fail = Fail {
        command: "/b_read".to_owned(),
        message: "no file\n".to_owned(),
        value: Some(2),
    };
    round_trip(
        &Reply::Fail(fail),
        json!({"address": "/fail", "args": {
            "command": "/b_read", "message": "no file\n", "value": 2
        }}),
    );
}

#[test]
fn a_mode_or_an_action_is_its_name() {
    for (mode, name) in [
        (ErrorMode::Off, "off"),
        (ErrorMode::On, "on"),
        (ErrorMode::BundleOff, "bundle_off"),
        (ErrorMode::BundleOn, "bundle_on"),
    ] {
        round_trip(
            &Command::Error(mode),
            json!({"address": "/error", "args": name}),
        );
    }
    round_trip(
        &Command::DumpOsc(DumpMode::ParsedAndHex),
        json!({"address": "/dumpOSC", "args": "parsed_and_hex"}),
    );
    let order = NOrder {
        action: OrderAction::Before,
        target: NodeId(1),
        nodes: vec![NodeId(2)],
    };
    round_trip(
        &Command::NOrder(order),
        json!({"address": "/n_order", "args": {"action": "before", "target": 1, "nodes": [2]}}),
    );
}

#[test]
fn every_fill_names_its_command() {
    let flags = WaveFillFlags {
        normalize: false,
        wavetable: true,
        clear: false,
    };
    let flags_json = json!({"normalize": false, "wavetable": true, "clear": false});
    for (fill, json) in [
        (
            BufferFill::Sine2 {
                flags,
                partials: vec![Partial {
                    frequency: 1.0,
                    amplitude: 0.5,
                }],
            },
            json!({"sine2": {"flags": flags_json, "partials": [
                {"frequency": 1.0, "amplitude": 0.5}
            ]}}),
        ),
        (
            BufferFill::Sine3 {
                flags,
                partials: vec![PhasedPartial {
                    frequency: 1.0,
                    amplitude: 0.5,
                    phase: 0.25,
                }],
            },
            json!({"sine3": {"flags": flags_json, "partials": [
                {"frequency": 1.0, "amplitude": 0.5, "phase": 0.25}
            ]}}),
        ),
        (
            BufferFill::Cheby {
                flags,
                amplitudes: vec![1.0],
            },
            json!({"cheby": {"flags": flags_json, "amplitudes": [1.0]}}),
        ),
        (
            BufferFill::Copy {
                to_frame: 0,
                source: BufferId::from(5),
                from_frame: 8,
                frames: -1,
            },
            json!({"copy": {"to_frame": 0, "source": 5, "from_frame": 8, "frames": -1}}),
        ),
        (
            BufferFill::Plugin {
                name: "normalize".to_owned(),
                args: vec![Argument::Float(1.0)],
            },
            json!({"plugin": {"name": "normalize", "args": [{"float": 1.0}]}}),
        ),
    ] {
        let generate = BGen {
            buffer: BufferId::from(4),
            fill,
        };
        round_trip(
            &Command::BGen(generate),
            json!({"address": "/b_gen", "args": {"buffer": 4, "fill": json}}),
        );
    }
}

#[test]
fn a_synth_of_a_tree_without_controls_has_null_controls() {
    let tree = TreeGroup {
        id: NodeId(0),
        children: vec![TreeNode::Synth(TreeSynth {
            id: NodeId(1000),
            def: "default".to_owned(),
            controls: (),
        })],
    };
    round_trip(
        &Reply::GQueryTreeReply(GQueryTreeReply::WithoutControls(tree)),
        json!({"address": "/g_queryTree.reply", "args": {"without_controls": {
            "id": 0, "children": [{"synth": {"id": 1000, "def": "default", "controls": null}}]
        }}}),
    );
}

#[test]
fn a_format_is_its_first_name_as_written() {
    let format = |name: &str| serde_json::from_value::<HeaderFormat>(json!(name)).ok();
    assert_eq!(format("wav"), Some(HeaderFormat::Wav));
    assert_eq!(format("WAV"), None);
    assert_eq!(format("wave"), None);
}

#[test]
fn a_float_that_is_not_finite_does_not_read_back() {
    let tr = Reply::Tr(Tr {
        node: NodeId(1000),
        id: 7,
        value: f32::NAN,
    });
    let json = serde_json::to_value(&tr).expect("serializes");
    assert_eq!(json["args"]["value"], serde_json::Value::Null);
    assert!(serde_json::from_value::<Reply>(json).is_err());
}

#[test]
fn serde_json_refuses_bundles_42_deep_and_trees_and_completions_43_deep() {
    let reads_back = |json: &serde_json::Value| -> bool {
        let text = json.to_string();
        serde_json::from_str::<serde_json::Value>(&text).is_ok()
    };
    let bundles = |depth| {
        let mut packet = Packet::Message(Command::Status);
        for _ in 0..depth {
            packet = Packet::Bundle(Bundle {
                time: Timetag::IMMEDIATELY,
                packets: vec![packet],
            });
        }
        serde_json::to_value(packet).expect("serializes")
    };
    let completions = |depth| {
        let mut command = Command::Status;
        for _ in 0..depth {
            command = Command::BFree(BFree {
                buffer: BufferId::from(0),
                completion: Some(Completion::Message(Box::new(command))),
            });
        }
        serde_json::to_value(command).expect("serializes")
    };
    let trees = |depth: i32| {
        let mut group = TreeGroup {
            id: NodeId(depth - 1),
            children: vec![],
        };
        for id in (0..depth - 1).rev() {
            group = TreeGroup {
                id: NodeId(id),
                children: vec![TreeNode::Group(group)],
            };
        }
        serde_json::to_value(Reply::GQueryTreeReply(GQueryTreeReply::WithoutControls(
            group,
        )))
        .expect("serializes")
    };
    assert!(reads_back(&bundles(41)));
    assert!(!reads_back(&bundles(42)));
    assert!(reads_back(&completions(42)));
    assert!(!reads_back(&completions(43)));
    assert!(reads_back(&trees(42)));
    assert!(!reads_back(&trees(43)));
}

#[test]
fn a_raw_message_is_its_address_and_its_typed_args() {
    let raw = RawMessage {
        address: "/demo".to_owned(),
        args: vec![
            RawArg::Int(-1),
            RawArg::Float(0.5),
            RawArg::Double(0.25),
            RawArg::String("default".to_owned()),
            RawArg::Blob(vec![1, 2]),
            RawArg::Array(vec![RawArg::Float(1.0), RawArg::String("c3".to_owned())]),
        ],
    };
    round_trip(
        &raw,
        json!({"address": "/demo", "args": [
            {"type": "i", "data": -1},
            {"type": "f", "data": 0.5},
            {"type": "d", "data": 0.25},
            {"type": "s", "data": "default"},
            {"type": "b", "data": [1, 2]},
            {"type": "[", "data": [{"type": "f", "data": 1.0}, {"type": "s", "data": "c3"}]}
        ]}),
    );
    let fraction = json!({"address": "/sync", "args": [{"type": "i", "data": 1.5}]});
    assert!(serde_json::from_value::<RawMessage>(fraction).is_err());
}

#[test]
fn a_message_of_the_pump_names_its_uuids() {
    let session = "5049297f-342b-4b8b-8c62-54edbb5d3335";
    let plugin = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";
    let uuid = |text: &str| text.parse().expect("a uuid");
    round_trip(
        &Command::LoadRoot(Root {
            session: uuid(session),
        }),
        json!({"address": "/loadRoot", "args": {"session": session}}),
    );
    round_trip(
        &Command::UnloadPlugin(Plugin {
            session: uuid(session),
            plugin: uuid(plugin),
        }),
        json!({"address": "/unloadPlugin", "args": {"session": session, "plugin": plugin}}),
    );
    let failure = PluginFailure {
        session: uuid(session),
        plugin: uuid(plugin),
        reason: "The root group is not ready.".to_owned(),
    };
    round_trip(
        &Reply::LoadPluginFailure(failure),
        json!({"address": "/loadPlugin.failure", "args": {
            "session": session, "plugin": plugin, "reason": "The root group is not ready."
        }}),
    );
}
