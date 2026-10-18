//! Conformance against a real scsynth: it reads each command as the crate
//! means it, and each of its replies decodes strictly. The README says how
//! to run these tests.

use std::env;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use rosc::OscType::{Float, Int, String as Text};
use rosc::{OscMessage, OscPacket, OscType};
use scsynth_osc::command::buffer::{
    BAlloc, BAllocRead, BAllocReadChannel, BFill, BFree, BGen, BGet, BGetn, BQuery, BRead,
    BReadChannel, BSet, BSetSampleRate, BSetn, BWrite, Channels, HeaderFormat, SampleFormat,
};
use scsynth_osc::command::buffer_fill::{BufferFill, Partial, PhasedPartial, WaveFillFlags};
use scsynth_osc::command::control_bus::{CFill, CGet, CGetn, CSet, CSetn};
use scsynth_osc::command::group::{
    GDumpTree, GFreeAll, GHead, GNew, GroupMove, NewGroup, TreeRequest,
};
use scsynth_osc::command::node::{
    ControlFill, ControlMapping, ControlMappingRange, ControlRange, NBefore, NFill, NFree, NMap,
    NMapn, NOrder, NQuery, NRun, NSet, NSetn, NTrace, NodeMove, NodeRun,
};
use scsynth_osc::command::synth::{RangeRequest, SGet, SGetn, SNew, SNoid};
use scsynth_osc::command::synth_def::{DFree, DLoad, DLoadDir, DRecv};
use scsynth_osc::command::top_level::{DumpMode, ErrorMode, Notify, SyncId};
use scsynth_osc::reply::command_replies::{BufferCommand, Done, Fail};
use scsynth_osc::reply::group::{GQueryTreeReply, TreeNode, TreeValue};
use scsynth_osc::reply::node_notification::{NodeKind, NodeState};
use scsynth_osc::{
    AddAction, ArrayElement, BufferId, BusMapping, Command, Completion, Control, ControlSetting,
    ControlValue, DecodeErrorKind, IndexFill, IndexRange, IndexValue, IndexValues, NewNodeId,
    NodeId, Number, OrderAction, Reply,
};

/// Messages of scsynth that the crate leaves out: see the README.
const LEFT_OUT: [&str; 1] = ["/d_removed"];

/// A client of the scsynth at `SCSYNTH_ADDR`. It keeps the replies that no
/// test asked for yet.
struct Server {
    socket: UdpSocket,
    pending: Vec<Reply>,
}

impl Server {
    fn connect() -> Self {
        let address = env::var("SCSYNTH_ADDR").expect("SCSYNTH_ADDR names the scsynth to test");
        let socket = UdpSocket::bind("127.0.0.1:0").expect("a local socket");
        socket.connect(&address).expect("the address of scsynth");
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("a timeout");
        Self {
            socket,
            pending: Vec::new(),
        }
    }

    /// A client that receives the notifications.
    fn registered() -> Self {
        let mut server = Self::connect();
        server.send(&Command::Notify(Notify {
            on: true,
            client_id: None,
        }));
        server.wait(|reply| matches!(reply, Reply::Done(Done::Notify { .. })).then_some(()));
        server
    }

    fn send(&self, command: &Command) {
        self.socket.send(&command.encode()).expect("a sent command");
    }

    fn send_raw(&self, address: &str, args: Vec<OscType>) {
        let message = OscPacket::Message(OscMessage {
            addr: address.to_owned(),
            args,
        });
        let bytes = rosc::encoder::encode(&message).expect("an OSC message");
        self.socket.send(&bytes).expect("a sent message");
    }

    /// The next reply that the crate covers, decoded strictly.
    fn next(&self) -> Reply {
        let mut buffer = vec![0; 65_536];
        loop {
            let size = self.socket.recv(&mut buffer).expect("a reply within 5 s");
            let bytes = &buffer[..size];
            match Reply::decode(bytes) {
                Ok(reply) => return reply,
                Err(error)
                    if *error.kind() == DecodeErrorKind::UnknownAddress
                        && LEFT_OUT.contains(&error.address()) => {}
                Err(error) => panic!("{error}: {:?}", rosc::decoder::decode_udp(bytes)),
            }
        }
    }

    /// The first reply that `want` takes, from the kept ones or the next.
    /// A `/fail` that it does not take fails the test.
    fn wait<T>(&mut self, mut want: impl FnMut(&Reply) -> Option<T>) -> T {
        if let Some((index, value)) = self
            .pending
            .iter()
            .enumerate()
            .find_map(|(index, reply)| want(reply).map(|value| (index, value)))
        {
            self.pending.remove(index);
            return value;
        }
        loop {
            let reply = self.next();
            if let Some(value) = want(&reply) {
                return value;
            }
            if let Reply::Fail(fail) = &reply {
                panic!("an unexpected {fail:?}");
            }
            self.pending.push(reply);
        }
    }

    /// Waits until scsynth did every command sent before.
    fn sync(&mut self) {
        static NEXT: AtomicI32 = AtomicI32::new(1);
        let id = SyncId(NEXT.fetch_add(1, Ordering::Relaxed));
        self.send(&Command::Sync(id));
        self.wait(|reply| (*reply == Reply::Synced(id)).then_some(()));
    }

    fn done(&mut self, done: &Done) {
        self.wait(|reply| matches!(reply, Reply::Done(found) if found == done).then_some(()));
    }

    /// The `/done` of a command whose completion message replied: it must
    /// come after that reply.
    fn done_after(&mut self, done: &Done) {
        let early = self
            .pending
            .iter()
            .any(|reply| matches!(reply, Reply::Done(found) if found == done));
        assert!(!early, "{done:?} came before the completion message");
        self.done(done);
    }

    fn state(&mut self, node: i32, notification: fn(&Reply) -> Option<&NodeState>) -> NodeState {
        self.wait(|reply| {
            notification(reply)
                .filter(|state| state.node == NodeId(node))
                .copied()
        })
    }

    /// Defines the synths of `SCSYNTH_SYNTHDEF`.
    fn define(&mut self) {
        self.send(&Command::DRecv(DRecv {
            data: std::fs::read(synthdef()).expect("the synth definition file"),
            completion: None,
        }));
        self.done(&Done::DRecv);
    }

    fn controls(&mut self, synth: i32, controls: &[&str]) -> Vec<f32> {
        self.send(&Command::SGet(SGet {
            synth: NodeId(synth),
            controls: controls.iter().map(|&name| Control::from(name)).collect(),
        }));
        self.wait(|reply| match reply {
            Reply::NSet(set) if set.node == NodeId(synth) => {
                Some(set.controls.iter().map(|reading| reading.value).collect())
            }
            _ => None,
        })
    }
}

/// The path of `default.scsyndef`, from the testsuite of SuperCollider.
fn synthdef() -> PathBuf {
    env::var("SCSYNTH_SYNTHDEF")
        .expect("SCSYNTH_SYNTHDEF names default.scsyndef")
        .into()
}

fn go(reply: &Reply) -> Option<&NodeState> {
    match reply {
        Reply::NGo(state) => Some(state),
        _ => None,
    }
}

fn end(reply: &Reply) -> Option<&NodeState> {
    match reply {
        Reply::NEnd(state) => Some(state),
        _ => None,
    }
}

fn moved(reply: &Reply) -> Option<&NodeState> {
    match reply {
        Reply::NMove(state) => Some(state),
        _ => None,
    }
}

fn group(id: i32, action: AddAction, target: i32) -> NewGroup {
    NewGroup {
        id: NewNodeId::new(id).expect("an ID of 0 or more"),
        action,
        target: NodeId(target),
    }
}

/// A synth of the definition `default`, silent: its amplitude is 0.
fn synth(id: i32, action: AddAction, target: i32) -> Command {
    Command::SNew(SNew {
        def: "default".to_owned(),
        id: NewNodeId::new(id).expect("an ID of 0 or more"),
        action,
        target: NodeId(target),
        controls: vec![ControlSetting::new("amp", 0.0)],
    })
}

#[test]
#[ignore = "needs a scsynth: see the README"]
fn the_top_level_and_synth_definition_commands() {
    let mut server = Server::connect();
    server.send(&Command::Notify(Notify {
        on: true,
        client_id: None,
    }));
    server.wait(|reply| matches!(reply, Reply::Done(Done::Notify { .. })).then_some(()));
    server.send(&Command::Status);
    server.wait(|reply| matches!(reply, Reply::StatusReply(_)).then_some(()));
    server.send(&Command::Version);
    let version = server.wait(|reply| match reply {
        Reply::VersionReply(version) => Some(version.clone()),
        _ => None,
    });
    assert_eq!((version.program.as_str(), version.major), ("scsynth", 3));
    server.send(&Command::RtMemoryStatus);
    server.wait(|reply| matches!(reply, Reply::RtMemoryStatusReply(_)).then_some(()));
    server.send(&Command::DumpOsc(DumpMode::Off));
    server.send(&Command::Error(ErrorMode::On));
    server.send(&Command::ClearSched);
    server.sync();

    server.send(&Command::DRecv(DRecv {
        data: std::fs::read(synthdef()).expect("the synth definition file"),
        completion: Some(Completion::Message(Box::new(Command::Status))),
    }));
    server.wait(|reply| matches!(reply, Reply::StatusReply(_)).then_some(()));
    server.done_after(&Done::DRecv);
    server.send(&Command::DLoad(DLoad {
        path: synthdef().display().to_string(),
        completion: None,
    }));
    server.done(&Done::DLoad);
    let directory = synthdef().parent().expect("a directory").to_owned();
    server.send(&Command::DLoadDir(DLoadDir {
        path: directory.display().to_string(),
        completion: None,
    }));
    server.done(&Done::DLoadDir);
    server.send(&Command::DFree(DFree {
        names: vec!["help_out".to_owned()],
    }));
    server.sync();

    server.send(&Command::Notify(Notify {
        on: false,
        client_id: None,
    }));
    server.done(&Done::NotifyOff);
}

#[test]
#[ignore = "needs a scsynth: see the README"]
fn the_node_synth_and_group_commands() {
    let mut server = Server::registered();
    server.define();
    create_nodes(&mut server);
    set_controls(&mut server);
    map_controls(&mut server);
    run_and_query(&mut server);
    move_nodes(&mut server);
    free_nodes(&mut server);
}

/// Group 2000 holds synth 2001 and group 2010.
fn create_nodes(server: &mut Server) {
    server.send(&Command::GNew(GNew {
        groups: vec![group(2000, AddAction::Head, 0)],
    }));
    let state = server.state(2000, go);
    assert_eq!(
        state.kind,
        NodeKind::Group {
            head: None,
            tail: None
        }
    );
    server.send(&Command::PNew(GNew {
        groups: vec![group(2010, AddAction::Tail, 2000)],
    }));
    server.state(2010, go);
    server.send(&synth(2001, AddAction::Head, 2000));
    assert_eq!(server.state(2001, go).parent, Some(NodeId(2000)));
}

fn set_controls(server: &mut Server) {
    server.send(&Command::NSet(NSet {
        node: NodeId(2001),
        controls: vec![
            ControlSetting::new("freq", 220.0),
            ControlSetting::new(
                3,
                ControlValue::Array(vec![ArrayElement::Float(0.5), ArrayElement::Int(1)]),
            ),
        ],
    }));
    assert_eq!(
        server.controls(2001, &["freq", "pan", "gate"]),
        [220.0, 0.5, 1.0]
    );
    server.send(&Command::SGetn(SGetn {
        synth: NodeId(2001),
        ranges: vec![RangeRequest {
            control: Control::Index(1),
            count: 2,
        }],
    }));
    let values = server.wait(|reply| match reply {
        Reply::NSetn(setn) if setn.node == NodeId(2001) => Some(setn.ranges[0].values.clone()),
        _ => None,
    });
    assert_eq!(values, [220.0, 0.0]);
    server.send(&Command::NSetn(NSetn {
        node: NodeId(2001),
        ranges: vec![ControlRange {
            control: Control::from("freq"),
            values: vec![Number::Float(110.0)],
        }],
    }));
    assert_eq!(server.controls(2001, &["freq"]), [110.0]);

    server.send(&Command::NFill(NFill {
        node: NodeId(2001),
        ranges: vec![ControlFill {
            control: Control::from("freq"),
            count: 1,
            value: Number::Float(330.0),
        }],
    }));
    assert_eq!(server.controls(2001, &["freq"]), [330.0]);
    server.send_raw(
        "/n_fill",
        vec![Int(2001), Text("freq".to_owned()), Int(1), Float(440.0)],
    );
    assert_eq!(server.controls(2001, &["freq"]), [0.0]);
}

fn map_controls(server: &mut Server) {
    // Mappings show in the tree with its controls.
    server.send(&Command::NMap(NMap {
        node: NodeId(2001),
        mappings: vec![ControlMapping {
            control: Control::from("freq"),
            bus: 5,
        }],
    }));
    // `amp` has one control in default.scsyndef.
    server.send(&Command::NMapan(NMapn {
        node: NodeId(2001),
        mappings: vec![ControlMappingRange {
            control: Control::from("amp"),
            bus: 6,
            count: 2,
        }],
    }));
    server.send(&Command::NSet(NSet {
        node: NodeId(2001),
        controls: vec![ControlSetting::new("gate", BusMapping::Control(7))],
    }));
    let values = tree_values(server, &["freq", "amp", "pan", "gate"]);
    assert_eq!(
        values,
        [
            TreeValue::Bus(BusMapping::Control(5)),
            TreeValue::Bus(BusMapping::Audio(6)),
            TreeValue::Float(0.5),
            TreeValue::Bus(BusMapping::Control(7)),
        ]
    );
    server.send(&Command::NMapan(NMapn {
        node: NodeId(2001),
        mappings: vec![ControlMappingRange {
            control: Control::Index(2),
            bus: 8,
            count: 2,
        }],
    }));
    let values = tree_values(server, &["amp", "pan"]);
    assert_eq!(
        values,
        [
            TreeValue::Bus(BusMapping::Audio(8)),
            TreeValue::Bus(BusMapping::Audio(9)),
        ]
    );
    server.send(&Command::NMapn(NMapn {
        node: NodeId(2001),
        mappings: vec![ControlMappingRange {
            control: Control::Index(1),
            bus: -1,
            count: 4,
        }],
    }));
    server.send(&Command::NMapa(NMap {
        node: NodeId(2001),
        mappings: vec![ControlMapping {
            control: Control::from("gate"),
            bus: -1,
        }],
    }));
    server.send(&Command::GQueryTree(GDumpTree {
        groups: vec![TreeRequest {
            group: NodeId(2000),
            controls: false,
        }],
    }));
    server.wait(|reply| match reply {
        Reply::GQueryTreeReply(GQueryTreeReply::WithoutControls(tree))
            if tree.id == NodeId(2000) =>
        {
            Some(())
        }
        _ => None,
    });
}

/// The values of controls of synth 2001, from the tree of group 2000 with
/// its controls.
fn tree_values(server: &mut Server, names: &[&str]) -> Vec<TreeValue> {
    server.send(&Command::GQueryTree(GDumpTree {
        groups: vec![TreeRequest {
            group: NodeId(2000),
            controls: true,
        }],
    }));
    let tree = server.wait(|reply| match reply {
        Reply::GQueryTreeReply(GQueryTreeReply::WithControls(tree)) if tree.id == NodeId(2000) => {
            Some(tree.clone())
        }
        _ => None,
    });
    let [TreeNode::Synth(synth_node), TreeNode::Group(parallel)] = tree.children.as_slice() else {
        panic!("a synth and a group in {tree:?}");
    };
    assert_eq!((synth_node.id, parallel.id), (NodeId(2001), NodeId(2010)));
    names
        .iter()
        .map(|&name| {
            synth_node
                .controls
                .iter()
                .find(|control| control.control == Control::from(name))
                .map_or_else(|| panic!("a control {name}"), |control| control.value)
        })
        .collect()
}

fn run_and_query(server: &mut Server) {
    server.send(&Command::NRun(NRun {
        nodes: vec![NodeRun {
            node: NodeId(2001),
            run: false,
        }],
    }));
    server.wait(|reply| match reply {
        Reply::NOff(state) if state.node == NodeId(2001) => Some(()),
        _ => None,
    });
    server.send(&Command::NRun(NRun {
        nodes: vec![NodeRun {
            node: NodeId(2001),
            run: true,
        }],
    }));
    server.wait(|reply| match reply {
        Reply::NOn(state) if state.node == NodeId(2001) => Some(()),
        _ => None,
    });
    server.send(&Command::NQuery(NQuery {
        nodes: vec![NodeId(2001)],
    }));
    server.wait(|reply| match reply {
        Reply::NInfo(state) if state.node == NodeId(2001) => Some(()),
        _ => None,
    });
}

fn move_nodes(server: &mut Server) {
    server.send(&synth(2002, AddAction::Tail, 2010));
    server.state(2002, go);
    server.send(&Command::NAfter(NBefore {
        moves: vec![NodeMove {
            node: NodeId(2002),
            target: NodeId(2001),
        }],
    }));
    assert_eq!(server.state(2002, moved).prev, Some(NodeId(2001)));
    server.send(&Command::NBefore(NBefore {
        moves: vec![NodeMove {
            node: NodeId(2002),
            target: NodeId(2001),
        }],
    }));
    assert_eq!(server.state(2002, moved).next, Some(NodeId(2001)));
    server.send(&Command::NOrder(NOrder {
        action: OrderAction::Head,
        target: NodeId(2000),
        nodes: vec![NodeId(2001), NodeId(2002)],
    }));
    assert_eq!(server.state(2002, moved).prev, Some(NodeId(2001)));
    server.send(&Command::GTail(GHead {
        moves: vec![GroupMove {
            group: NodeId(2010),
            node: NodeId(2002),
        }],
    }));
    assert_eq!(server.state(2002, moved).parent, Some(NodeId(2010)));
    server.send(&Command::GHead(GHead {
        moves: vec![GroupMove {
            group: NodeId(2000),
            node: NodeId(2002),
        }],
    }));
    assert_eq!(server.state(2002, moved).prev, None);
}

fn free_nodes(server: &mut Server) {
    server.send(&Command::NTrace(NTrace {
        nodes: vec![NodeId(2001)],
    }));
    server.send(&Command::GDumpTree(GDumpTree {
        groups: vec![TreeRequest {
            group: NodeId(2000),
            controls: true,
        }],
    }));
    server.send(&Command::SNoid(SNoid {
        synths: vec![NodeId(2002)],
    }));
    server.sync();
    server.send(&Command::NFree(NFree {
        nodes: vec![NodeId(2001)],
    }));
    server.state(2001, end);
    server.send(&synth(2003, AddAction::Head, 2010));
    server.state(2003, go);
    server.send(&Command::GDeepFree(GFreeAll {
        groups: vec![NodeId(2000)],
    }));
    server.state(2003, end);
    server.send(&Command::GFreeAll(GFreeAll {
        groups: vec![NodeId(2000)],
    }));
    server.state(2010, end);
    server.send(&Command::NFree(NFree {
        nodes: vec![NodeId(2000)],
    }));
    server.state(2000, end);
}

#[test]
#[ignore = "needs a scsynth: see the README"]
fn the_buffer_commands() {
    let mut server = Server::connect();
    let buffer = BufferId::from(30);
    allocate(&mut server, buffer);
    set_samples(&mut server, buffer);
    fill(&mut server, buffer);
    read_and_write_files(&mut server, buffer);
}

fn buffer_done(command: BufferCommand, buffer: BufferId) -> Done {
    Done::Buffer { command, buffer }
}

fn allocate(server: &mut Server, buffer: BufferId) {
    server.send(&Command::BAlloc(BAlloc {
        buffer,
        frames: 64,
        channels: 1,
        completion: Some(Completion::Message(Box::new(Command::BQuery(BQuery {
            buffers: vec![buffer],
        })))),
        sample_rate: 22_050.0,
    }));
    let info = server.wait(|reply| match reply {
        Reply::BInfo(info) => Some(info.buffers.clone()),
        _ => None,
    });
    assert_eq!(
        (
            info[0].buffer,
            info[0].frames,
            info[0].channels,
            info[0].sample_rate
        ),
        (buffer, 64, 1, 22_050.0)
    );
    server.done_after(&buffer_done(BufferCommand::BAlloc, buffer));
}

fn set_samples(server: &mut Server, buffer: BufferId) {
    server.send(&Command::BSet(BSet {
        buffer,
        samples: vec![IndexValue {
            index: 3,
            value: 0.5,
        }],
    }));
    server.send(&Command::BSetn(BSetn {
        buffer,
        ranges: vec![IndexValues {
            start: 4,
            values: vec![0.25, 0.75],
        }],
    }));
    server.send(&Command::BFill(BFill {
        buffer,
        ranges: vec![IndexFill {
            start: 10,
            count: 3,
            value: 1.0,
        }],
    }));
    server.send(&Command::BGet(BGet {
        buffer,
        samples: vec![3],
    }));
    let samples = server.wait(|reply| match reply {
        Reply::BSet(set) if set.buffer == buffer => Some(set.samples.clone()),
        _ => None,
    });
    assert_eq!(
        samples,
        [IndexValue {
            index: 3,
            value: 0.5
        }]
    );
    server.send(&Command::BGetn(BGetn {
        buffer,
        ranges: vec![
            IndexRange { start: 4, count: 2 },
            IndexRange {
                start: 10,
                count: 3,
            },
        ],
    }));
    let ranges = server.wait(|reply| match reply {
        Reply::BSetn(setn) if setn.buffer == buffer => Some(setn.ranges.clone()),
        _ => None,
    });
    assert_eq!(ranges[0].values, [0.25, 0.75]);
    assert_eq!(ranges[1].values, [1.0, 1.0, 1.0]);

    server.send(&Command::BSetSampleRate(BSetSampleRate {
        buffer,
        sample_rate: 0.0,
    }));
    server.send(&Command::BQuery(BQuery {
        buffers: vec![buffer],
    }));
    let info = server.wait(|reply| match reply {
        Reply::BInfo(info) => Some(info.buffers.clone()),
        _ => None,
    });
    assert!(
        (info[0].sample_rate - 22_050.0).abs() > 1.0,
        "the rate of the server"
    );
}

fn fill(server: &mut Server, buffer: BufferId) {
    let flags = WaveFillFlags {
        normalize: true,
        wavetable: false,
        clear: true,
    };
    for fill in [
        BufferFill::Sine1 {
            flags,
            amplitudes: vec![1.0, 0.5],
        },
        BufferFill::Sine2 {
            flags,
            partials: vec![Partial {
                frequency: 1.5,
                amplitude: 1.0,
            }],
        },
        BufferFill::Sine3 {
            flags,
            partials: vec![PhasedPartial {
                frequency: 2.0,
                amplitude: 1.0,
                phase: 0.5,
            }],
        },
        BufferFill::Cheby {
            flags,
            amplitudes: vec![1.0, 0.5],
        },
        BufferFill::Copy {
            to_frame: 0,
            source: buffer,
            from_frame: 32,
            frames: 8,
        },
        BufferFill::Plugin {
            name: "normalize".to_owned(),
            args: vec![scsynth_osc::Argument::Float(0.5)],
        },
    ] {
        server.send(&Command::BGen(BGen { buffer, fill }));
        server.done(&buffer_done(BufferCommand::BGen, buffer));
    }
}

/// Writes the buffer into a file, reads it back, and frees the buffers.
fn read_and_write_files(server: &mut Server, buffer: BufferId) {
    let path = env::temp_dir().join(format!("scsynth-osc-{}.wav", std::process::id()));
    let path_text = path.display().to_string();
    server.send(&Command::BWrite(BWrite {
        buffer,
        path: path_text.clone(),
        header: HeaderFormat::Wav,
        sample: SampleFormat::Float,
        frames: -1,
        buffer_frame: 0,
        leave_open: false,
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BWrite, buffer));
    let other = BufferId::from(31);
    server.send(&Command::BAllocRead(BAllocRead {
        buffer: other,
        path: path_text.clone(),
        file_frame: 0,
        frames: 0,
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BAllocRead, other));
    server.send(&Command::BAllocReadChannel(BAllocReadChannel {
        buffer: other,
        path: path_text.clone(),
        file_frame: 0,
        frames: 16,
        channels: Channels::new(vec![0]).expect("one channel"),
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BAllocReadChannel, other));
    server.send(&Command::BQuery(BQuery {
        buffers: vec![other],
    }));
    let info = server.wait(|reply| match reply {
        Reply::BInfo(info) => Some(info.buffers.clone()),
        _ => None,
    });
    assert_eq!((info[0].frames, info[0].channels), (16, 1));
    server.send(&Command::BRead(BRead {
        buffer,
        path: path_text.clone(),
        file_frame: 0,
        frames: -1,
        buffer_frame: 0,
        leave_open: false,
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BRead, buffer));
    server.send(&Command::BReadChannel(BReadChannel {
        buffer,
        path: path_text,
        file_frame: 0,
        frames: -1,
        buffer_frame: 0,
        leave_open: true,
        channels: Channels::new(vec![0]).expect("one channel"),
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BReadChannel, buffer));
    server.send(&Command::BClose(BFree {
        buffer,
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BClose, buffer));
    server.send(&Command::BZero(BFree {
        buffer,
        completion: None,
    }));
    server.done(&buffer_done(BufferCommand::BZero, buffer));
    for buffer in [buffer, other] {
        server.send(&Command::BFree(BFree {
            buffer,
            completion: None,
        }));
        server.done(&Done::Buffer {
            command: BufferCommand::BFree,
            buffer,
        });
    }
    std::fs::remove_file(path).expect("the written file");
}

#[test]
#[ignore = "needs a scsynth: see the README"]
fn the_control_bus_commands() {
    let mut server = Server::connect();
    server.send(&Command::CSet(CSet {
        buses: vec![
            IndexValue {
                index: 40,
                value: Number::Float(0.5),
            },
            IndexValue {
                index: 41,
                value: Number::Int(3),
            },
        ],
    }));
    server.send(&Command::CSetn(CSetn {
        ranges: vec![IndexValues {
            start: 42,
            values: vec![Number::Float(0.25), Number::Int(2)],
        }],
    }));
    server.send(&Command::CFill(CFill {
        ranges: vec![IndexFill {
            start: 44,
            count: 2,
            value: Number::Float(0.75),
        }],
    }));
    server.send(&Command::CGet(CGet {
        buses: vec![40, 41],
    }));
    let buses = server.wait(|reply| match reply {
        Reply::CSet(set) => Some(set.buses.clone()),
        _ => None,
    });
    assert_eq!(
        buses,
        [
            IndexValue {
                index: 40,
                value: 0.5
            },
            IndexValue {
                index: 41,
                value: 3.0
            }
        ]
    );
    server.send(&Command::CGetn(CGetn {
        ranges: vec![IndexRange {
            start: 42,
            count: 4,
        }],
    }));
    let ranges = server.wait(|reply| match reply {
        Reply::CSetn(setn) => Some(setn.ranges.clone()),
        _ => None,
    });
    assert_eq!(
        ranges,
        [IndexValues {
            start: 42,
            values: vec![0.25, 2.0, 0.75, 0.75]
        }]
    );
}

#[test]
#[ignore = "needs a scsynth: see the README"]
fn a_failure_decodes() {
    let mut server = Server::connect();
    server.send(&Command::NFree(NFree {
        nodes: vec![NodeId(999_999)],
    }));
    let fail = server.wait(|reply| match reply {
        Reply::Fail(fail) => Some(fail.clone()),
        _ => None,
    });
    assert_eq!(
        fail,
        Fail {
            command: "/n_free".to_owned(),
            message: "Node 999999 not found".to_owned(),
            value: None
        }
    );
}
