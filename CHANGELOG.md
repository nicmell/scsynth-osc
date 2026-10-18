# Changelog

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
