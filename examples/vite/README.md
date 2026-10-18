# The scsynth-osc playground

A Vite page that runs scsynth-osc in the browser, compiled to wasm. It
shows one OSC message of scsynth in three forms, and keeps them in step:

- Typed: the serde form of the library's `Command` or `Reply`, such as
  `{"address": "/sync", "args": 7}`. A switch picks which of the two.
- Raw: the library's `RawMessage`, such as
  `{"address": "/sync", "args": [{"type": "i", "data": 7}]}`.
- Bytes: the OSC packet, in hex.

Edit any pane, and the other two follow. A pane that the library refuses
shows the error, and keeps its last content, dimmed. For example, a raw
`/sync` with an `f` argument encodes as raw, but the typed pane shows
`/sync: argument 0 has the type 'f', it must be an int`. The menu holds
sample commands and replies.

The page talks to no scsynth: a browser cannot send UDP.

## Run it

You need Rust with the target `wasm32-unknown-unknown`, wasm-pack, and
Node 22.12 or newer. In this folder:

```sh
npm install
npm run wasm   # wasm-pack builds wasm/ into pkg/
npm run dev    # or: npm run build, then npm run preview
```

## How it works

- `wasm/` is a small crate of its own. It exports six functions with
  wasm-bindgen. `encodeCommand`, `encodeReply` and `encodeRaw` take a JS
  object and give the bytes, as a `Uint8Array`. `decodeCommand`,
  `decodeReply` and `decodeRaw` do the reverse. serde-wasm-bindgen turns
  the library's serde form into JS objects and back. An error is a JS
  `Error` with the library's message.
- `wasm-pack build --target web` makes an ES module that loads the
  `.wasm` through `new URL(…, import.meta.url)`, so Vite needs no plugin.
- `src/main.ts` holds the panes, and `src/examples.ts` the samples.
- A JS number holds no difference between `440` and `440.0`. Where the
  library takes an int or a float, such as a control value, a whole
  number reads as an int: the raw pane shows `i`, not `f`.
