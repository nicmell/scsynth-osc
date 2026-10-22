# Changelog

## Branch sc-app3 (unreleased)

### Added

- The messages of sc-app3's pump: `/loadRoot`, `/unloadRoot`,
  `/loadPlugin` and `/unloadPlugin`, and their `.success` and `.failure`
  answers, with their sessions and plugins as `uuid::Uuid`. Their types are
  in the modules `command::custom` and `reply::custom`, and
  `command::custom::ADDRESSES` lists the addresses of the commands. See
  [The protocol of sc-app3's pump](README.md#the-protocol-of-sc-app3s-pump).
- The clock of a session: `/clock.ping` and its `/clock.pong` of five
  numbers, and the reply of its clock synth, `/tick`, with the last click
  on its probe bus.
- The streams of a session: `/stream.open`, `/stream.close` and the
  answers of an open. `/stream.frames` carries the frames of a stream, as
  a blob of little-endian `f32`.

## 0.1.2 (2026-10-19)

### Added

- `RawArg` has a variant for each OSC type that rosc reads: `Long` (`h`),
  `Time` (`t`), `Char` (`c`), `Color` (`r`), `Midi` (`m`), `True` (`T`),
  `False` (`F`), `Nil` (`N`) and `Inf` (`I`). `Color` and `MidiMessage`
  are new structs, with the fields of rosc's. With the feature `serde`,
  the `type` of each one is its OSC tag.

### Changed

- `RawMessage::decode` and a `Packet<RawMessage>` no longer refuse an
  argument of those types. `Command` and `Reply` decode as strictly as
  before.

## 0.1.1 (2026-10-18)

### Added

- `RawMessage` and `RawArg`: an OSC message as its address and its typed
  arguments. `Command` and `Reply` convert to and from it with `to_raw`
  and `from_raw`, and a `Packet<RawMessage>` decodes bundles. See
  [Raw messages](README.md#raw-messages).
- With the feature `serde`, a raw message is
  `{"address": …, "args": [{"type": …, "data": …}]}`, and a blob
  serializes as bytes, through `serde_bytes`.

## 0.1.0 (2026-10-18)

The first release. It holds the commands and the replies of the Server
Command Reference that scsynth 3.14.1 implements, as Rust types that
encode and decode strictly.

### Known issues

The README describes each of these:

- Decoding does not limit how deep completion messages, bundles and
  arrays nest, and it takes the malformed packets that rosc takes. See
  [Limits](README.md#limits).
- Encoding takes negative counts and frames, and large counts, that make
  scsynth write outside its memory or stall. See [Traps](README.md#traps).
