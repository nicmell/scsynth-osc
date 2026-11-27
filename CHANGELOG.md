# Changelog

## Branch sc-app3 (unreleased)

### Changed

- The messages of sc-app3's pump name no session, because the websocket of
  a page belongs to one session. `/loadRoot`, `/unloadRoot` and their
  `.success` answers take no argument.

### Added

- The messages of sc-app3's pump: `/loadRoot`, `/unloadRoot`,
  `/loadPlugin` and `/unloadPlugin`, and their `.success` and `.failure`
  answers, with their plugins as `uuid::Uuid`. Their types are
  in the modules `command::custom` and `reply::custom`, and
  `command::custom::ADDRESSES` lists the addresses of the commands. See
  [The protocol of sc-app3's pump](README.md#the-protocol-of-sc-app3s-pump).
- The clock of a session: `/clock.ping` and its `/clock.pong` with server
  time, the beat of the pump and the latency of the session. The reply of
  its clock synth, `/tick`, carries the count and the phase of a tick.
- The streams of a session: `/stream.open`, `/stream.close` and the
  answers of an open. `/stream.frames` carries the frames of a stream, as
  a blob of little-endian `f32`.
- The synth definitions of a page: `/loadSynthdef` with the bytes of a
  synth definition file, `/unloadSynthdef`, and the `.success` and
  `.failure` answers of both, which name the definition. A name takes 1 to
  255 bytes, and the bytes cannot be empty.
- The nodes of the elements of a plugin: `/loadGroup` and `/loadSynth`,
  with the id of the element and the id of its parent, and for a synth its
  definition and its controls. Their `.success` carries the node, and
  their `.failure` a reason. An id cannot be empty.
- The unloads of those nodes: `/unloadGroup` and `/unloadSynth`, with the
  id of the element, and the `.success` and `.failure` answers of both.
- The Dirt of an element of a plugin: `/loadDirt`, and its `.success` and
  `.failure` answers. `/loadDirt` carries the number of orbits and the
  first output bus of the Dirt. `/dirt.play` carries a sound at a cycle,
  with its index, its span and the pairs of its parameters. Its only
  answer is `/dirt.play.failure`. An index is 0 or more, and the name of a
  parameter cannot be empty.
- The tempo of a session: `/getTempo` and `/setTempo`. The `.success` of
  `/getTempo` carries the cycles per second, a cycle and its server time.
  `/setTempo` carries the cycles per second, and the pump sends
  `/tempoChange` with the new tempo to every page.
- The controls of the synth of an element of a plugin: `/setControl`,
  with controls as in `/loadSynth`, and `/getControl`, with ranges as in
  `/s_getn`. Neither gets an answer of its own. The pump sends `/control`
  to every page, with the ranges and the values of an `/n_setn` of
  scsynth. An id cannot be empty, and each message carries one control or
  more.

### Removed

- `/loadRoot`, `/unloadRoot`, their four answers and `RootFailure`: the
  pump of sc-app3 keeps the root group of its session by itself.
  `command::custom::ADDRESSES` lists 17 addresses.

## 0.2.1 (2026-11-22)

### Added

- With the feature `serde`, `RAW_TYPESCRIPT`: the TypeScript declarations
  of `RawMessage` and `RawArg`, as the JavaScript values that
  serde-wasm-bindgen writes, for the `typescript_custom_section` of a
  wasm-bindgen crate. See [Features](README.md#features).

## 0.2.0 (2026-11-08)

### Changed

- With the feature `serde`, `RawArg::Array` is the sequence of its
  arguments, such as `[{"type": "i", "data": 1}]`, in the place of
  `{"type": "[", "data": […]}`, which no longer deserializes. A sequence
  is always an array, so the pair `["i", 1]` no longer reads as an int.
- A `RawArg` deserializes only from a format that describes its own
  types, such as JSON.

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
