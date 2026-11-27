//! The commands of sc-app3's pump, which never reach scsynth, one module
//! per family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod clock;
mod dirt;
mod groups;
mod streams;
mod tempo;

pub use clock::ClockPing;
pub use dirt::{DirtParameter, DirtPlay, DirtValue, LoadDirt};
pub use groups::Plugin;
pub use streams::{Stream, StreamOpen};
pub use tempo::SetTempo;

/// The addresses of the commands of the pump. The pump reads them, and
/// never passes them on to scsynth.
pub const ADDRESSES: [&str; 9] = [
    "/loadPlugin",
    "/unloadPlugin",
    "/clock.ping",
    "/stream.open",
    "/stream.close",
    "/loadDirt",
    "/dirt.play",
    "/getTempo",
    "/setTempo",
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
        let dirt = LoadDirt {
            plugin: other,
            node: 2,
            block: 0,
            orbits: 1,
            out: 0,
        };
        let sound = DirtPlay {
            plugin: other,
            node: 2,
            cycle: 0.0,
            index: 0,
            span: 1.0,
            parameters: vec![DirtParameter::new("s", "bd")],
        };
        let commands = [
            Command::LoadPlugin(plugin),
            Command::UnloadPlugin(plugin),
            Command::ClockPing(ClockPing { sent_ms: 0.0 }),
            Command::StreamOpen(open),
            Command::StreamClose(stream),
            Command::LoadDirt(dirt),
            Command::DirtPlay(sound),
            Command::GetTempo,
            Command::SetTempo(SetTempo { cps: 0.5 }),
        ];
        assert_eq!(commands.map(|command| command.address()), ADDRESSES);
    }
}
