//! Top-level commands. `/quit`, `/status`, `/clearSched`, `/version` and
//! `/rtMemoryStatus` take no arguments, so they have no type of their own
//! here: see [`Command`](crate::Command).

use crate::argument::Argument;
use crate::codec::{Payload, Reader, Writer, coded};
use crate::error::DecodeError;

/// The arguments of `/notify`. The server replies:
/// - `/done /notify clientID maxLogins` once the sender is registered;
/// - `/done /notify` once its registration ends;
/// - `/fail /notify` when the sender is already registered, when the server
///   has `maxLogins` clients, or when an unregistered sender asks to end.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Notify {
    /// Whether the sender receives notifications from now on.
    pub on: bool,
    /// The client ID that the sender asks for. -1 asks for none, as `None`
    /// does. When another client has the ID, scsynth gives the sender a free
    /// one, and no error.
    pub client_id: Option<i32>,
}

impl Payload for Notify {
    fn write(&self, writer: &mut Writer) {
        writer.flag(self.on);
        if let Some(client_id) = self.client_id {
            writer.int(client_id);
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            on: reader.flag()?,
            client_id: reader.optional_int()?,
        })
    }
}

/// The arguments of `/cmd`: a command that a plugin defines. scsynth reads
/// the name only, and hands the rest to the plugin. Whether a plugin
/// replies, and how, is up to the plugin.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cmd {
    /// The name of the command, as the plugin registered it.
    pub name: String,
    /// The arguments for the plugin.
    pub args: Vec<Argument>,
}

impl Payload for Cmd {
    fn write(&self, writer: &mut Writer) {
        writer.string(&self.name);
        for arg in &self.args {
            writer.push(arg.to_osc());
        }
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            name: reader.string()?,
            args: read_plugin_args(reader)?,
        })
    }
}

/// Every argument left, as plugin arguments.
pub(crate) fn read_plugin_args(reader: &mut Reader<'_>) -> Result<Vec<Argument>, DecodeError> {
    let mut args = Vec::new();
    while reader.remains() {
        let value = reader.any("a plugin argument")?;
        let arg = Argument::from_osc(value)
            .ok_or_else(|| reader.wrong(value, "an int, a float, a double, a string or a blob"))?;
        args.push(arg);
    }
    Ok(args)
}

coded! {
    /// What `/dumpOSC` makes the server print for each message it receives.
    /// See [Differences](crate#differences-from-the-reference).
    DumpMode {
        /// Nothing.
        Off = 0,
        /// The message, parsed.
        Parsed = 1,
        /// The bytes of the message, in hexadecimal.
        Hex = 2,
        /// Both.
        ParsedAndHex = 3,
    }
}

impl Payload for DumpMode {
    fn write(&self, writer: &mut Writer) {
        writer.int(self.code());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.coded("a dump mode, 0 to 3", Self::from_code)
    }
}

/// The ID of a `/sync`, which its `/synced` reply carries back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct SyncId(pub i32);

impl Payload for SyncId {
    fn write(&self, writer: &mut Writer) {
        writer.int(self.0);
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.int().map(Self)
    }
}

coded! {
    /// What `/error` does to the posting of errors. See
    /// [Differences](crate#differences-from-the-reference).
    ErrorMode {
        /// Turns posting off until a `/error 1`.
        Off = 0,
        /// Turns posting on.
        On = 1,
        /// Turns posting off for the rest of the bundle.
        BundleOff = -1,
        /// Undoes one [`ErrorMode::BundleOff`] in the bundle.
        BundleOn = -2,
    }
}

impl Payload for ErrorMode {
    fn write(&self, writer: &mut Writer) {
        writer.int(self.code());
    }

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.coded("an error mode, 0, 1, -1 or -2", Self::from_code)
    }
}

#[cfg(test)]
mod tests {
    use rosc::OscType::{Blob, Bool, Double, Float, Int, String as Text};

    use super::*;
    use crate::Command;
    use crate::error::DecodeErrorKind;
    use crate::testing::{assert_wire, decode_error};

    #[test]
    fn the_top_level_commands_have_their_wire_form() {
        assert_wire(&Command::Quit, "/quit", vec![]);
        let register = Notify {
            on: true,
            client_id: None,
        };
        assert_wire(&Command::Notify(register), "/notify", vec![Int(1)]);
        let leave = Notify {
            on: false,
            client_id: Some(3),
        };
        assert_wire(&Command::Notify(leave), "/notify", vec![Int(0), Int(3)]);
        assert_wire(&Command::Status, "/status", vec![]);
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
        let wire = vec![
            Text("demo".to_owned()),
            Int(1),
            Float(0.5),
            Double(0.25),
            Text("x".to_owned()),
            Blob(vec![1, 2]),
        ];
        assert_wire(&Command::Cmd(cmd), "/cmd", wire);
        assert_wire(
            &Command::DumpOsc(DumpMode::ParsedAndHex),
            "/dumpOSC",
            vec![Int(3)],
        );
        assert_wire(&Command::Sync(SyncId(7)), "/sync", vec![Int(7)]);
        assert_wire(&Command::ClearSched, "/clearSched", vec![]);
        assert_wire(
            &Command::Error(ErrorMode::BundleOn),
            "/error",
            vec![Int(-2)],
        );
        assert_wire(&Command::Version, "/version", vec![]);
        assert_wire(&Command::RtMemoryStatus, "/rtMemoryStatus", vec![]);
    }

    #[test]
    fn coded_values_outside_the_reference_are_refused() {
        for (address, code) in [("/notify", 2), ("/dumpOSC", 4), ("/error", 2)] {
            let kind = decode_error::<Command>(address, vec![Int(code)]);
            assert!(
                matches!(kind, DecodeErrorKind::Invalid { index: 0, .. }),
                "{address}: {kind:?}"
            );
        }
    }

    #[test]
    fn a_float_is_no_int_although_scsynth_would_truncate_it() {
        assert_eq!(
            decode_error::<Command>("/sync", vec![Float(7.0)]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "an int",
                found: 'f'
            }
        );
    }

    #[test]
    fn arguments_too_many_or_too_few_are_refused() {
        assert_eq!(
            decode_error::<Command>("/status", vec![Int(1)]),
            DecodeErrorKind::Trailing { index: 0 }
        );
        assert_eq!(
            decode_error::<Command>("/sync", vec![]),
            DecodeErrorKind::Missing {
                index: 0,
                expected: "an int"
            }
        );
        assert_eq!(
            decode_error::<Command>("/notify", vec![Int(1), Int(2), Int(3)]),
            DecodeErrorKind::Trailing { index: 2 }
        );
    }

    #[test]
    fn a_plugin_argument_of_another_type_is_refused() {
        assert_eq!(
            decode_error::<Command>("/cmd", vec![Text("demo".to_owned()), Bool(true)]),
            DecodeErrorKind::WrongType {
                index: 1,
                expected: "an int, a float, a double, a string or a blob",
                found: 'T'
            }
        );
    }

    #[test]
    fn an_unknown_address_is_refused() {
        assert_eq!(
            decode_error::<Command>("/beat", vec![]),
            DecodeErrorKind::UnknownAddress
        );
    }
}
