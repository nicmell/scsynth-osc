# scsynth-osc

The OSC protocol of scsynth, the synthesis server of SuperCollider. This
crate holds the commands and the replies of the
[Server Command Reference](https://doc.sccode.org/Reference/Server-Command-Reference.html)
as Rust types. They encode to the bytes that scsynth reads, and decode
from the bytes that it writes.

```rust
use scsynth_osc::{Command, Reply};
use scsynth_osc::command::top_level::SyncId;

// A client asks scsynth to answer once its pending work is done.
let bytes = Command::Sync(SyncId(7)).encode();
assert_eq!(Command::decode(&bytes), Ok(Command::Sync(SyncId(7))));

// scsynth answers /synced 7.
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
| Top-level | `/quit`, `/notify`, `/status`, `/cmd`, `/dumpOSC`, `/sync`, `/clearSched`, `/error`, `/version`, `/rtMemoryStatus` | `/status.reply`, `/version.reply`, `/rtMemoryStatus.reply`, `/synced` |
| Synth definitions | `/d_recv`, `/d_load`, `/d_loadDir`, `/d_free` | |
| Replies to commands | | `/done`, `/fail` |

A command without arguments is a unit variant, such as `Command::Quit`.
The arguments of any other command are a type of its section, named
after its address, such as `command::synth_def::DLoad` for `/d_load`. A
command with one argument takes a type of its own instead:
`Command::Sync(SyncId)`, `Command::Error(ErrorMode)` and
`Command::DumpOsc(DumpMode)`.

## Strictness

- Encoding writes each argument in the one OSC type that the reference
  gives it. The crate leaves an optional argument out only at the end of
  a message. If a later argument is there, the crate writes the earlier
  ones with the defaults of scsynth.
- Decoding takes each argument only in that type. scsynth itself reads a
  float in place of an int, and truncates it. This crate refuses it.
- Decoding refuses a message with an argument too many or too few, and a
  value that the protocol does not know.
- No value of these types encodes to bytes that make scsynth misread a
  message, or hang.
- The crate checks the wire and the coded values, such as add actions,
  modes and formats. It does not model the server: whether a node or a
  buffer exists is not its concern.

### Completion messages

An asynchronous command can carry a completion message: commands that the
server runs once the work is done, before it replies `/done`.

- It is a `Completion`: one command, or a bundle of commands. A bundle
  inside it is refused, because scsynth fails on one.
- It travels as a blob. An empty blob is refused, because scsynth leaves
  it unread.
- In the place of a completion message, scsynth skips one int, and sclang
  sends 0 there for no completion message. The crate decodes that 0 as no
  completion message, and never writes it.

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
- `/d_removed`, the replies of SendReply and `/diskin` are messages of
  scsynth that the reference does not list.
- supernova, the other server of SuperCollider, differs in places, and
  this crate does not follow it.

## Features

- `serde`: `Serialize` and `Deserialize` for every type. A command or a
  reply is then an object with its `address` and its `args`, such as
  `{"address": "/sync", "args": 7}`.

## License

None yet. The crate is not published.
