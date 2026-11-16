//! The commands of sc-app3's pump, which never reach scsynth, one module
//! per family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod clock;
mod groups;
mod nodes;
mod states;
mod streams;
mod synthdefs;

pub use clock::ClockPing;
pub use groups::Plugin;
pub use nodes::{LoadGroup, LoadSynth, Node};
pub use states::SetState;
pub use streams::{Stream, StreamOpen};
pub use synthdefs::{LoadSynthdef, Synthdef};

pub(crate) use nodes::read_id;
pub(crate) use synthdefs::read_name;

/// The addresses of the commands of the pump. The pump reads them, and
/// never passes them on to scsynth.
pub const ADDRESSES: [&str; 14] = [
    "/loadRoot",
    "/unloadRoot",
    "/loadPlugin",
    "/unloadPlugin",
    "/clock.ping",
    "/stream.open",
    "/stream.close",
    "/loadSynthdef",
    "/unloadSynthdef",
    "/loadGroup",
    "/loadSynth",
    "/unloadGroup",
    "/unloadSynth",
    "/setState",
];

use uuid::Uuid;

use crate::codec::{Reader, Writer};
use crate::error::DecodeError;

pub(crate) fn write_uuid(writer: &mut Writer, uuid: Uuid) {
    writer.string(&uuid.hyphenated().to_string());
}

/// A uuid, as the string that [`write_uuid`] writes.
pub(crate) fn read_uuid(reader: &mut Reader<'_>) -> Result<Uuid, DecodeError> {
    let index = reader.index();
    let text = reader.string()?;
    Uuid::try_parse(&text)
        .ok()
        .filter(|uuid| uuid.hyphenated().to_string() == text)
        .ok_or_else(|| {
            reader.invalid(
                index,
                format!("{text:?} is not a uuid in lowercase with hyphens"),
            )
        })
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::Command;

    #[test]
    fn the_addresses_of_the_pump_are_those_of_its_commands() {
        let other = Uuid::from_u128(2);
        let plugin = Plugin { plugin: other };
        let stream = Stream { stream: other };
        let open = StreamOpen {
            stream: other,
            bus: 256,
            channels: 1,
        };
        let synthdef = Synthdef {
            plugin: other,
            name: "a1b2/sine".to_owned(),
        };
        let load = LoadSynthdef {
            plugin: other,
            name: synthdef.name.clone(),
            data: vec![1],
        };
        let group = LoadGroup {
            plugin: other,
            id: "a".to_owned(),
            parent: String::new(),
        };
        let synth = LoadSynth {
            plugin: other,
            id: "b".to_owned(),
            parent: "a".to_owned(),
            def: "c".to_owned(),
            controls: Vec::new(),
        };
        let node = Node {
            plugin: other,
            id: "a".to_owned(),
        };
        let state = SetState {
            plugin: other,
            key: "a".to_owned(),
            value: "1".to_owned(),
        };
        let commands = [
            Command::LoadRoot,
            Command::UnloadRoot,
            Command::LoadPlugin(plugin),
            Command::UnloadPlugin(plugin),
            Command::ClockPing(ClockPing { sent_ms: 0.0 }),
            Command::StreamOpen(open),
            Command::StreamClose(stream),
            Command::LoadSynthdef(load),
            Command::UnloadSynthdef(synthdef),
            Command::LoadGroup(group),
            Command::LoadSynth(synth),
            Command::UnloadGroup(node.clone()),
            Command::UnloadSynth(node),
            Command::SetState(state),
        ];
        assert_eq!(commands.map(|command| command.address()), ADDRESSES);
    }
}
