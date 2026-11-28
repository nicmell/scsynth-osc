//! An enum of messages that a crate outside the library declares with
//! `messages!`, as an app does.

use scsynth_osc::{
    Bundle, Command, DecodeError, DecodeErrorKind, NodeId, Packet, Payload, RawArg, RawMessage,
    Reader, Timetag, Writer,
};

/// The arguments of `/hello`: a node, a name that cannot be empty, and a
/// level.
#[derive(Clone, Debug, PartialEq)]
struct Hello {
    node: NodeId,
    name: String,
    level: f64,
}

impl Payload for Hello {
    fn write(&self, writer: &mut Writer) {
        self.node.write(writer);
        writer.string(&self.name);
        writer.double(self.level);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let node = NodeId::read(reader)?;
        let index = reader.index();
        let name = reader.string()?;
        if name.is_empty() {
            return Err(reader.invalid(index, "an empty name"));
        }
        Ok(Self {
            node,
            name,
            level: reader.double()?,
        })
    }
}

/// The argument of `/value`: an int or a string.
#[derive(Clone, Debug, PartialEq)]
enum Value {
    Int(i32),
    Text(String),
}

impl Payload for Value {
    fn write(&self, writer: &mut Writer) {
        match self {
            Self::Int(value) => writer.int(*value),
            Self::Text(value) => writer.string(value),
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        const EXPECTED: &str = "an int or a string";
        match reader.arg(EXPECTED)? {
            RawArg::Int(value) => Ok(Self::Int(value)),
            RawArg::String(value) => Ok(Self::Text(value)),
            other => Err(reader.wrong(&other, EXPECTED)),
        }
    }
}

scsynth_osc::messages! {
    /// The commands of an app.
    enum AppCommand {
        /// `/hello`: a greeting.
        "/hello" as HELLO => Hello(Hello),
        /// `/value`: a value.
        "/value" as VALUE => Value(Value),
        /// `/bye`: no arguments.
        "/bye" as BYE => Bye,
    }
}

fn hello() -> AppCommand {
    AppCommand::Hello(Hello {
        node: NodeId(1001),
        name: "ada".to_owned(),
        level: 0.5,
    })
}

fn every_command() -> [AppCommand; 4] {
    [
        hello(),
        AppCommand::Value(Value::Int(7)),
        AppCommand::Value(Value::Text("x".to_owned())),
        AppCommand::Bye,
    ]
}

fn raw(address: &str, args: Vec<RawArg>) -> RawMessage {
    RawMessage {
        address: address.to_owned(),
        args,
    }
}

/// The kind of error that decoding `address` with `args` gives.
fn decode_error(address: &str, args: Vec<RawArg>) -> DecodeErrorKind {
    let error = AppCommand::from_raw(&raw(address, args)).expect_err("the message does not decode");
    assert_eq!(error.address(), address);
    error.kind().clone()
}

#[test]
fn an_enum_of_an_app_has_a_constant_per_address() {
    assert_eq!(AppCommand::HELLO, "/hello");
    assert_eq!(AppCommand::ADDRESSES, &["/hello", "/value", "/bye"]);
    let addresses: Vec<_> = every_command().iter().map(AppCommand::address).collect();
    assert_eq!(addresses, ["/hello", "/value", "/value", "/bye"]);
}

#[test]
fn a_message_of_an_app_round_trips() {
    for message in every_command() {
        assert_eq!(AppCommand::decode(&message.encode()).as_ref(), Ok(&message));
        assert_eq!(
            AppCommand::from_raw(&message.to_raw()).as_ref(),
            Ok(&message)
        );
        assert_eq!(message.to_raw().encode(), message.encode());
    }
    let args = vec![
        RawArg::Int(1001),
        RawArg::String("ada".to_owned()),
        RawArg::Double(0.5),
    ];
    assert_eq!(hello().to_raw(), raw("/hello", args));
    let bundle = Packet::Bundle(Bundle {
        time: Timetag::IMMEDIATELY,
        packets: every_command().map(Packet::Message).into(),
    });
    assert_eq!(Packet::<AppCommand>::decode(&bundle.encode()), Ok(bundle));
}

#[test]
fn a_message_of_an_app_decodes_strictly() {
    assert_eq!(
        decode_error("/s_new", vec![]),
        DecodeErrorKind::UnknownAddress
    );
    let status = AppCommand::decode(&Command::Status.encode()).expect_err("not of the app");
    assert_eq!(status.kind(), &DecodeErrorKind::UnknownAddress);
    let ours = Command::decode(&hello().encode()).expect_err("not of scsynth");
    assert_eq!(ours.kind(), &DecodeErrorKind::UnknownAddress);
    assert!(matches!(
        decode_error("/hello", vec![RawArg::Int(1), RawArg::Float(0.5)]),
        DecodeErrorKind::WrongType {
            index: 1,
            expected: "a string",
            found: 'f',
            ..
        }
    ));
    assert!(matches!(
        decode_error("/value", vec![RawArg::Double(1.0)]),
        DecodeErrorKind::WrongType {
            index: 0,
            expected: "an int or a string",
            found: 'd',
            ..
        }
    ));
    let empty = vec![
        RawArg::Int(1),
        RawArg::String(String::new()),
        RawArg::Double(0.5),
    ];
    assert!(matches!(
        decode_error("/hello", empty),
        DecodeErrorKind::Invalid { index: 1, .. }
    ));
    assert!(matches!(
        decode_error("/hello", vec![RawArg::Int(1)]),
        DecodeErrorKind::Missing { index: 1, .. }
    ));
    assert!(matches!(
        decode_error("/bye", vec![RawArg::Int(1)]),
        DecodeErrorKind::Trailing { index: 0, .. }
    ));
}

#[test]
fn a_raw_address_ends_at_its_first_nul_byte_as_for_command() {
    assert_eq!(
        Command::from_raw(&raw("/status\0x", vec![])),
        Ok(Command::Status)
    );
    assert_eq!(
        AppCommand::from_raw(&raw("/bye\0x", vec![])),
        Ok(AppCommand::Bye)
    );
    let unknown = AppCommand::from_raw(&raw("/nothing\0x", vec![])).expect_err("unknown");
    assert_eq!(unknown.address(), "/nothing");
    assert_eq!(unknown.kind(), &DecodeErrorKind::UnknownAddress);
}

#[cfg(feature = "serde")]
mod with_serde {
    use scsynth_osc::{DecodeError, Payload, Reader, Writer};
    use serde_json::json;

    /// The arguments of `/tempo`: the cycles per second.
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Tempo {
        cps: f64,
    }

    impl Payload for Tempo {
        fn write(&self, writer: &mut Writer) {
            writer.double(self.cps);
        }

        fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
            Ok(Self {
                cps: reader.double()?,
            })
        }
    }

    scsynth_osc::messages! {
        serde;
        /// The answers of an app, with serde.
        enum AppReply {
            /// `/tempo`: the tempo.
            "/tempo" as TEMPO => Tempo(Tempo),
            /// `/ready`: no arguments.
            "/ready" as READY => Ready,
        }
    }

    #[test]
    fn an_enum_declared_with_serde_has_the_shape_of_command() {
        let tempo = AppReply::Tempo(Tempo { cps: 0.5 });
        let json = json!({"address": "/tempo", "args": {"cps": 0.5}});
        assert_eq!(serde_json::to_value(&tempo).expect("serializes"), json);
        let back: AppReply = serde_json::from_value(json).expect("deserializes");
        assert_eq!(back, tempo);
        assert_eq!(
            serde_json::to_value(AppReply::Ready).expect("serializes"),
            json!({"address": "/ready"})
        );
        assert_eq!(AppReply::ADDRESSES, &[AppReply::TEMPO, AppReply::READY]);
    }
}
