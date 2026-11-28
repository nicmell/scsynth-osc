# Changelog

## 0.2.2 (2026-11-28)

### Added

- Each address is a constant of its enum, such as `Command::S_NEW` for
  `/s_new` and `Reply::STATUS_REPLY` for `/status.reply`, and
  `ADDRESSES` lists the addresses of an enum. See
  [The messages](README.md#the-messages).
- `messages!`, the macro that declares `Command` and `Reply`: a crate
  declares an enum of its own messages with it, and with
  `messages! { serde; … }` for `Serialize` and `Deserialize`. `Payload`,
  `Reader` and `Writer` are public for the arguments of those messages,
  and the value types, such as `NodeId`, implement `Payload`. See
  [Custom messages](README.md#custom-messages).

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
