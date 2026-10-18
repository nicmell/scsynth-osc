# scsynth-osc

The OSC protocol of scsynth, the synthesis server of SuperCollider. This
crate holds the commands and the replies of the
[Server Command Reference](https://doc.sccode.org/Reference/Server-Command-Reference.html)
as Rust types. They encode to the bytes that scsynth reads, and decode
from the bytes that it writes.

```rust
use scsynth_osc::command::synth::SNew;
use scsynth_osc::{AddAction, Command, ControlSetting, NewNodeId, NodeId, Reply, SyncId};

// A synth of the definition "default", at the head of group 1.
let new = Command::SNew(SNew {
    def: "default".to_owned(),
    id: NewNodeId::AUTO,
    action: AddAction::Head,
    target: NodeId(1),
    controls: vec![ControlSetting::new("freq", 440.0)],
});
let bytes = new.encode();
assert_eq!(Command::decode(&bytes), Ok(new));

// scsynth answers /sync 7 with /synced 7.
let answer = Reply::Synced(SyncId(7)).encode();
assert_eq!(Reply::decode(&answer), Ok(Reply::Synced(SyncId(7))));
```

The crate follows scsynth 3.14.1. Where the code of scsynth and the
reference differ, the crate does what scsynth does, and this file lists
the difference.

## The messages

`Command` has one variant per command of the reference, and `Reply` one
per message of the reference that scsynth sends, except the messages
under [Left out](#left-out). Each encodes to one OSC message and decodes
from one. A `Packet` adds bundles. The modules `command` and `reply`
follow the sections of the reference.

| Section | Commands | Replies |
| --- | --- | --- |
| Top-level | `/quit` `/notify` `/status` `/cmd` `/dumpOSC` `/sync` `/clearSched` `/error` `/version` `/rtMemoryStatus` | `/status.reply` `/version.reply` `/rtMemoryStatus.reply` `/synced` |
| Synth definitions | `/d_recv` `/d_load` `/d_loadDir` `/d_free` | |
| Nodes | `/n_free` `/n_run` `/n_set` `/n_setn` `/n_fill` `/n_map` `/n_mapn` `/n_mapa` `/n_mapan` `/n_before` `/n_after` `/n_query` `/n_trace` `/n_order` | |
| Synths | `/s_new` `/s_get` `/s_getn` `/s_noid` | `/n_set` `/n_setn` |
| Groups | `/g_new` `/p_new` `/g_head` `/g_tail` `/g_freeAll` `/g_deepFree` `/g_dumpTree` `/g_queryTree` | `/g_queryTree.reply` |
| Unit generators | `/u_cmd` | |
| Buffers | `/b_alloc` `/b_allocRead` `/b_allocReadChannel` `/b_read` `/b_readChannel` `/b_write` `/b_free` `/b_zero` `/b_set` `/b_setn` `/b_setSampleRate` `/b_fill` `/b_gen` `/b_close` `/b_query` `/b_get` `/b_getn` | `/b_info` `/b_set` `/b_setn` |
| Control buses | `/c_set` `/c_setn` `/c_fill` `/c_get` `/c_getn` | `/c_set` `/c_setn` |
| Replies to commands | | `/done` `/fail` |
| Node notifications | | `/n_go` `/n_end` `/n_off` `/n_on` `/n_move` `/n_info` |
| Trigger notifications | | `/tr` |
| Buffer fills of `/b_gen` | `sine1` `sine2` `sine3` `cheby` `copy` | |

A command without arguments is a unit variant, such as `Command::Quit`.
The arguments of any other message are a type of its section, named after
its address, such as `command::synth_def::DLoad` for `/d_load`.

- Messages with the same arguments share one type, and the other names
  are aliases. `command::node::NMapa` is `NMap`, and
  `reply::buffer::BSet` is `command::buffer::BSet`.
- The node notifications share `reply::node_notification::NodeState`.
- A command with one argument takes a type of its own instead:
  `Command::Sync(SyncId)`, `Command::Error(ErrorMode)` and
  `Command::DumpOsc(DumpMode)`.
- The values that many messages share are at the root of the crate, such
  as `NodeId`, `AddAction` and `ControlValue`.
- The buffer and bus messages share `IndexValue`, `IndexValues`,
  `IndexFill` and `IndexRange`. Their values are `f32`, or `Number` where
  the reference takes an int too.

## Strictness

- Encoding writes each argument in the one OSC type that the reference
  gives it. It writes every argument that has a default in scsynth,
  except the client ID of `/notify`. That ID is an `Option`, and the
  crate leaves out a `None`. scsynth reads -1, its default, as no ID.
- Decoding takes each argument only in that type. scsynth itself reads a
  float in place of an int, and truncates it. This crate refuses it.
- Decoding gives an optional argument that the message leaves out the
  default of scsynth. It refuses a message with an argument too many or
  too few, and a value that the protocol does not know.
- The crate checks the wire and the coded values, such as add actions,
  modes and formats. It does not model the server: whether a node or a
  buffer exists is not its concern.

Except for the traps below, no value of these types encodes to bytes that
make scsynth misread a message, hang, or write outside its memory:

- A string ends at its first NUL byte, because scsynth reads the bytes
  after it as the next arguments. The crate writes no more of it.
- A `BufferId` is 0 or more. For a negative buffer, scsynth acts on buffer
  0, and an asynchronous command writes before the start of an array
  (`SC_World.h:119-129`, `SC_SequencedCommand.cpp:304-310`).
- A read takes at most 32 channels. scsynth writes a 33rd channel past the
  end of its array (`SC_SequencedCommand.cpp:663-671`).
- The controls of `/s_new` and `/n_set` follow the rules below.

### Completion messages

An asynchronous command can carry a completion message: commands that the
server runs once the work is done, before it replies `/done`. A bundle
with a time tag other than 0 or 1 is scheduled instead, and can run after
`/done`.

- It is a `Completion`: one command, or a bundle of commands. A bundle
  inside it is refused, because scsynth fails on one.
- It travels as a blob. An empty blob is refused, because scsynth leaves
  it unread.
- In the place of a completion message, scsynth skips one int, and sclang
  sends 0 there for no completion message. The crate decodes that 0 as no
  completion message, and never writes it. `/b_allocReadChannel` and
  `/b_readChannel` read every int there as a channel.
- Only `/b_alloc` has an argument after its completion message: the
  sample rate.

### Controls

`/s_new` and `/n_set` set controls by index or by name. A value is an
int, a float, a bus mapping or an array.

- A bus mapping travels as the string `c<index>` for a control bus or
  `a<index>` for an audio bus. Decoding takes only these forms. scsynth
  reads `cx` as bus 0, and `c` alone as no mapping. It reads the index as
  an `i32`, so an index above `i32::MAX` undoes the mapping.
- An array sets the control and the controls after it. It holds ints,
  floats and bus mappings, and no array: scsynth reads a nested array
  differently in `/s_new` and in `/n_set`, and any other type in an array
  makes it loop forever.

### Round trips

The bytes of a value decode back to the value, except for these values:

- A string with a NUL byte. The crate writes it up to that byte.
- A `DRecv` without bytes. Decoding refuses its empty blob.
- A `Done::Plugin` or a `BufferFill::Plugin` with the name of another
  variant. Its bytes decode as that variant, or do not decode.
- A `NodeState` with `Some(NodeId(-1))`. -1 travels for no node, and
  decodes as `None`.
- A `/g_queryTree.reply` deeper than 128 groups. Decoding refuses it.

### Traps

The crate does not rule these out:

- A bundle that holds both messages and bundles: scsynth 3.14.1 copies
  the first message over the header of the bundle, and misreads the rest
  (`SC_ComPort.cpp:84-91`). Send messages and bundles in bundles of their
  own.
- A negative count in a range of `/s_getn`, `/b_getn` or `/c_getn`, or
  counts whose sum passes `i32::MAX`: scsynth sizes its reply by the sum
  before it checks the ranges, and writes outside its memory
  (`SC_MiscCmds.cpp:1546-1564`, `common/scsynthsend.h:111-121`).
- A negative `buffer_frame` in `/b_read`, `/b_readChannel` or `/b_write`:
  scsynth reads the file into the memory before the buffer, or writes
  that memory to the file (`SC_SequencedCommand.cpp:623`, `915-924`,
  `1040`).
- A large count in a range of `/n_fill`, `/n_mapn` or `/n_mapan`: scsynth
  steps through the count on its audio thread, and does nothing else
  meanwhile (`SC_MiscCmds.cpp:331-405`, `601-658`). `/n_fill` of a named
  control with a count of `i32::MAX` took 35 seconds on an Apple M5.

These depend on the state of the server, so the crate cannot check them:

- A buffer number at or above the number of buffers of the server, set by
  `-b`: scsynth acts on buffer 0, or past the end of its arrays
  (`SC_World.h:119-129`).
- A range of `/b_setn` that does not fit in the buffer: scsynth reads the
  values after the part inside the buffer as the next ranges
  (`SC_MiscCmds.cpp:1379-1407`).
- `/b_readChannel` into a buffer with fewer channels than the read takes:
  scsynth writes past the end of the buffer
  (`SC_SequencedCommand.cpp:856-945`).

### Limits

- Decoding refuses a `/g_queryTree.reply` deeper than 128 groups, which
  bounds the stack that it takes.
- Decoding does not limit how deep completion messages, bundles and
  arrays nest. A deep nesting overflows the stack, and the process
  aborts. On a stack of 1 MiB, about 1,000 nested completion messages or
  bundles overflow a release build. That is about 20 KB of input. A debug
  build overflows at fewer than 50 completion messages.
- Nested completion messages take memory quadratic in their size. 65 KB
  of them, about the largest UDP datagram, took 250 MB to decode.
- rosc, the OSC decoder of the crate, takes some malformed packets:
  - An array that is not closed drops the arguments before it.
    `/sync ,i[i 5 7` decodes as `/sync 7`.
  - In a nested bundle, an element that does not decode drops itself and
    every element after it.
  - The bytes of a bundle element after its message are ignored.
- A reply with a string that is not UTF-8, such as a path in `/fail`,
  does not decode.
- Encoding panics on a list of more than `i32::MAX` values, which no OSC
  packet holds.

## Differences from the reference

- `/notify` always replies `/done /notify clientID maxLogins` to a
  registration, also without a client ID in the command. The crate also
  decodes the reply without `maxLogins`, which the reference allows. A second
  registration fails with `notify: already registered`, and the existing
  ID (`SC_SequencedCommand.cpp:1241-1282`).
- `/error -1` and `/error -2` count levels in a bundle: -1 turns posting
  off one level deeper, and -2 undoes one -1. -2 cannot turn posting on
  while `/error 0` turned it off. The levels end with the message or the
  bundle. `Command not found`, and the failures of the background stages
  of asynchronous commands, are sent in every mode
  (`SC_Lib.cpp:184-203`).
- `/dumpOSC` reads its mode as bits. The parsed form leaves out
  `/status`, and the hex dump does not (`SC_OscUtils.hpp:176-192`).
- `/d_recv` replies `/done` also to bytes that hold no valid synth
  definition. `/d_load` and `/d_loadDir` fail with the path as the whole
  message (`SC_SequencedCommand.cpp:1320-1467`).
- `/n_set` takes the bus mappings of `/s_new` (`SC_MiscCmds.cpp:407-470`).
- `/n_fill` reads a fourth argument in each range, and sets the controls
  to it, or to 0 at the end of the message. The range of the reference
  thus sets its controls to 0. The crate writes the value twice, and
  decodes a range only with the same value twice
  (`SC_MiscCmds.cpp:600-658`).
- `/p_new` creates a plain group (`SC_MiscCmds.cpp:998-1002`).
- `/g_new` takes the ID of a group that is already in the group where the
  new one goes, except with `AddAction::Replace`. It leaves that group in
  its place, and sends `/n_go` for it again (`SC_MiscCmds.cpp:898-996`).
- If `/b_alloc` cannot allocate the buffer, also for a size of 0, it
  sends no reply (`SC_SequencedCommand.cpp:292-302`).
- If the file stays open, `/b_write` still writes its frames. For
  `DiskOut`, write 0 frames (`SC_SequencedCommand.cpp:1009-1055`).
- The `copy` fill of `/b_gen` counts frames, not samples
  (`OscUGens.cpp:3492-3532`).
- `/version.reply` sends the commit hash at the length that git chooses,
  not always seven digits.
- The messages of the background stages of asynchronous commands end with
  a newline, except the paths of `/d_load` and `/d_loadDir`. The crate
  keeps them verbatim.

## Left out

- `/nrt_end` is in the reference, and `/n_cmd` in its command numbers,
  but scsynth 3.14.1 implements neither: the server answers
  `Command not found`.
- `/late` is in the reference, but scsynth 3.14.1 never sends it.
- The command numbers of the reference. scsynth takes the number of a
  command, as an int, in the place of its address. The crate writes and
  reads addresses only.
- `/none`, `/d_freeAll` and `/s_newargs` are in scsynth. The reference
  lists only their command numbers.
- An address without its leading slash, such as `status`. scsynth takes
  it as the address with the slash.
- These forms are in scsynth but not in the reference:
  - A node path in the place of a node ID, such as `h` for the head of a
    group.
  - A bus mapping as the fourth argument of a range of `/n_fill`.
  - Bus mappings in `/n_setn`.
- The fills `normalize`, `wnormalize`, `cantorFill`, `env` and
  `PreparePartConv` are in scsynth but not in the reference. They are
  plugin fills, by name.
- `/d_removed`, the replies of `SendReply` and `/diskin` are messages of
  scsynth that the reference does not list.
- supernova, the other server of SuperCollider, differs in places, and
  this crate does not follow it.

## Tests

The unit tests check the wire form of every message, and
`cargo test --all-features` runs them all.

`tests/conformance.rs` checks the crate against a real scsynth. The
server must read each command as the crate means it, and each reply must
decode strictly. These tests are ignored by default. Run them against a
scsynth that no other client uses:

```sh
scsynth -u 57210 -B 127.0.0.1 &
SCSYNTH_ADDR=127.0.0.1:57210 \
SCSYNTH_SYNTHDEF=<supercollider>/testsuite/server/supernova/default.scsyndef \
cargo test --test conformance -- --ignored
```

The tests use the nodes from 2000, the buffers 30 and 31 and the control
buses from 40. They write a sound file in the temporary directory, and
remove it.

## Features

- `serde`: `Serialize` and `Deserialize` for every type. A command or a
  reply is then an object with its `address` and its `args`, such as
  `{"address": "/sync", "args": 7}`.
  - The values that take more than one type, such as `Control` and
    `Number`, are untagged enums. They deserialize only from a format that
    describes its own types, such as JSON.
  - `serde_json` writes a NaN or an infinite float as `null`, and does not
    read it back.
  - `serde_json` reads at most 128 levels of nesting. Bundles nested 42
    deep, and trees and completion messages nested 43 deep, do not read
    back.

## License

Licensed under either of the Apache License, Version 2.0
(`LICENSE-APACHE`) or the MIT license (`LICENSE-MIT`), at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
