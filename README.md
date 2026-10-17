# scsynth-osc

The OSC protocol of scsynth, the synthesis server of SuperCollider. This
crate holds the commands and the replies of the
[Server Command Reference](https://doc.sccode.org/Reference/Server-Command-Reference.html)
as Rust types. They encode to the bytes that scsynth reads, and decode
from the bytes that it writes.

```rust
use scsynth_osc::command::synth::SNew;
use scsynth_osc::command::top_level::SyncId;
use scsynth_osc::{AddAction, Command, ControlSetting, NewNodeId, NodeId, Reply};

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

`Command` has one variant per command, and `Reply` one per message that
scsynth sends. Each encodes to one OSC message and decodes from one. A
`Packet` adds bundles. The modules `command` and `reply` follow the
sections of the reference.

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
  gives it. It writes every argument that has a default in scsynth. An
  optional argument without a default is an `Option`, such as the client
  ID of `/notify`. The crate leaves out a `None`.
- Decoding takes each argument only in that type. scsynth itself reads a
  float in place of an int, and truncates it. This crate refuses it.
- Decoding gives an optional argument that the message leaves out the
  default of scsynth. It refuses a message with an argument too many or
  too few, and a value that the protocol does not know.
- The crate checks the wire and the coded values, such as add actions,
  modes and formats. It does not model the server: whether a node or a
  buffer exists is not its concern.

No value of these types encodes to bytes that make scsynth misread a
message, hang, or write outside its memory:

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
server runs once the work is done, before it replies `/done`.

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
  reads `cx` as bus 0, and `c` alone as no mapping.
- An array sets the control and the controls after it. It holds ints,
  floats and bus mappings, and no array: scsynth reads a nested array
  differently in `/s_new` and in `/n_set`, and any other type in an array
  makes it loop forever.

### Traps that depend on the server

The crate cannot rule these out, because they depend on the state of the
server:

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

- Decoding refuses a `/g_queryTree.reply` deeper than 128 groups.
- Encoding panics on a list of more than `i32::MAX` values, which no OSC
  packet holds.

## Where scsynth 3.14.1 and the reference differ

- `/notify` always replies `/done /notify clientID maxLogins` to a
  registration, also without a client ID in the command. A second
  registration fails with `notify: already registered`, and the existing
  ID (`SC_SequencedCommand.cpp:1241-1282`).
- `/error -2` undoes one `/error -1` in a bundle. It cannot turn posting
  on while `/error 0` turned it off. `Command not found`, and the failures
  of the background stages of asynchronous commands, are sent in every
  mode (`SC_Lib.cpp:184-203`).
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
  new one goes. It sends `/n_go` for that group again
  (`SC_MiscCmds.cpp:898-996`).
- If `/b_alloc` cannot allocate the buffer, also for a size of 0, it
  sends no reply (`SC_SequencedCommand.cpp:292-302`).
- If the file stays open, `/b_write` still writes its frames. For
  `DiskOut`, write 0 frames (`SC_SequencedCommand.cpp:1009-1055`).
- The `copy` fill of `/b_gen` counts frames, not samples
  (`OscUGens.cpp:3492-3532`).
- `/version.reply` sends the commit hash at the length that git chooses,
  not always seven digits.
- The messages of the background stages of asynchronous commands end with
  a newline. The crate keeps them verbatim.

## Left out

- `/nrt_end` is in the reference, but scsynth 3.14.1 does not implement
  it: the server answers `Command not found`.
- `/late` is in the reference, but scsynth 3.14.1 never sends it.
- `/d_freeAll`, `/s_newargs` and the command numbers are in scsynth but
  not in the reference.
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

## Features

- `serde`: `Serialize` and `Deserialize` for every type. A command or a
  reply is then an object with its `address` and its `args`, such as
  `{"address": "/sync", "args": 7}`.

## License

None yet. The crate is not published.
