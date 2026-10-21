# scsynth-osc

A Rust crate for the OSC protocol of scsynth, the synthesis server of
SuperCollider. It holds the commands and the replies of the Server Command
Reference as Rust types, which encode and decode strictly. scsynth 3.14.1
is the reference. This file is the operating manual.

The README is the reference of the crate: the messages, the strictness,
the traps, the limits and the differences from the reference. Read it
before any work.

## Branches

- `main` is the library. Each release is a tag `v<version>`.
- `sc-app3` adds the protocol of the pump of sc-app3
  (github.com/nicmell/sc-app3). sc-app3 takes this branch as a git
  submodule. A change to that protocol goes to `sc-app3` only. Its
  messages live in `src/command/custom/` and `src/reply/custom/`, one
  file per family, and a new command also goes in
  `command::custom::ADDRESSES`.
- `sc-app3` takes `main` by a merge, never by a rebase or a force-push.
  The commits of sc-app3 pin commits of this branch. A rewrite orphans
  them.

## Ground rules

- A fact about scsynth comes from its source or from a measurement. The
  SuperCollider source is in `~/Projects/supercollider`, at the tag
  `Version-3.14.1`. `scsynth` 3.14.1 is on the PATH.
- Where the code of scsynth and the reference differ, the crate does what
  scsynth does. The README lists each difference.
- Never touch the scsynth on UDP 57110. The conformance tests run against
  a scsynth of their own: see the section Tests of the README.
- Every dependency must be at least 7 days old.

## Commands

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features
cargo check --all-features --target wasm32-unknown-unknown
cargo +1.85.0 test --all-features    # the MSRV
cargo deny check
```

CI (`.github/workflows/ci.yml`) runs them with `--locked`. It also runs
`cargo package` and builds the Vite example, whose README gives its own
commands (`examples/vite/README.md`). On the branch `sc-app3`, CI runs on
its pushes too.

## Conventions

- Edition 2024, MSRV 1.85, clippy pedantic, no unsafe code, docs on every
  public item. CI runs clippy with `-D warnings`.
- A comment states a fact that the code cannot: a measured number, a trap,
  a constraint. Rationale goes in the README.
- One fact, one owner. The owner is the test that breaks with the fact.
- Work goes on a branch. Every commit leaves fmt, clippy, the tests and the
  docs green. A branch reaches `main` through a GitHub PR.
- A release adds its section to `CHANGELOG.md` and gets its tag.
