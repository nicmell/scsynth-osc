import './style.css'
import init, {
  decodeCommand,
  decodeRaw,
  decodeReply,
  encodeCommand,
  encodeRaw,
  encodeReply,
} from '../pkg/scsynth_osc_wasm.js'
import { examples, type Kind } from './examples.ts'

/** The raw form of a message: its address, and its arguments in order. */
interface RawMessage {
  address: string
  args: RawArg[]
}

type RawArg =
  | { type: 'i' | 'f' | 'd'; data: number }
  | { type: 'h'; data: number | bigint }
  | { type: 's' | 'c'; data: string }
  | { type: 'b'; data: Uint8Array | number[] }
  | { type: 't'; data: { seconds: number; fraction: number } }
  | { type: 'r'; data: { red: number; green: number; blue: number; alpha: number } }
  | { type: 'm'; data: { port: number; status: number; data1: number; data2: number } }
  | { type: 'T' | 'F' | 'N' | 'I' }
  | { type: '['; data: RawArg[] }

type Pane = 'typed' | 'raw' | 'bytes'

await init()

const area = (pane: Pane) => document.querySelector<HTMLTextAreaElement>(`#${pane}`)!
const error = (pane: Pane) => document.querySelector<HTMLElement>(`#${pane}-error`)!
const menu = document.querySelector<HTMLSelectElement>('#example')!
const kinds = [...document.querySelectorAll<HTMLInputElement>('input[name=kind]')]

let kind: Kind = 'command'

/** `value` as indented JSON, with a byte array as a list of numbers. */
function json(value: unknown): string {
  return JSON.stringify(
    value,
    (_key, item: unknown) => (item instanceof Uint8Array ? Array.from(item) : item),
    2,
  )
}

/** `bytes` in hex: words of 4 bytes, 4 words per line. */
function hex(bytes: Uint8Array): string {
  const lines: string[] = []
  for (let line = 0; line < bytes.length; line += 16) {
    const words: string[] = []
    for (let word = line; word < Math.min(line + 16, bytes.length); word += 4) {
      const digits = Array.from(bytes.subarray(word, word + 4), (byte) =>
        byte.toString(16).padStart(2, '0'),
      )
      words.push(digits.join(''))
    }
    lines.push(words.join(' '))
  }
  return lines.join('\n')
}

function parseHex(text: string): Uint8Array {
  const digits = text.replace(/\s+/g, '')
  if (!/^(?:[0-9a-fA-F]{2})*$/.test(digits)) {
    throw new Error('The bytes are not pairs of hex digits.')
  }
  return Uint8Array.from(digits.match(/../g) ?? [], (pair) => parseInt(pair, 16))
}

function report(pane: Pane, fault: unknown) {
  error(pane).textContent = fault instanceof Error ? fault.message : String(fault)
}

/** Shows `content` in `pane`, or the error that it gave. A pane that
 * shows an error keeps its last content, dimmed. */
function fill(pane: Pane, content: () => string) {
  try {
    area(pane).value = content()
    area(pane).classList.remove('stale')
    error(pane).textContent = ''
  } catch (fault) {
    area(pane).classList.add('stale')
    report(pane, fault)
  }
}

function typed(bytes: Uint8Array): unknown {
  return kind === 'command' ? decodeCommand(bytes) : decodeReply(bytes)
}

/** The bytes of what `pane` holds. */
function bytesOf(pane: Pane): Uint8Array {
  const text = area(pane).value
  switch (pane) {
    case 'bytes':
      return parseHex(text)
    case 'raw':
      return encodeRaw(JSON.parse(text) as RawMessage)
    case 'typed':
      return kind === 'command' ? encodeCommand(JSON.parse(text)) : encodeReply(JSON.parse(text))
  }
}

/** Derives the other two panes from the one that changed. */
function edited(pane: Pane) {
  let bytes: Uint8Array
  try {
    bytes = bytesOf(pane)
  } catch (fault) {
    report(pane, fault)
    return
  }
  area(pane).classList.remove('stale')
  error(pane).textContent = ''
  if (pane !== 'bytes') fill('bytes', () => hex(bytes))
  if (pane !== 'raw') fill('raw', () => json(decodeRaw(bytes) as RawMessage))
  if (pane !== 'typed') fill('typed', () => json(typed(bytes)))
}

function choose(next: Kind) {
  kind = next
  for (const radio of kinds) radio.checked = radio.value === next
}

function load(index: number) {
  const example = examples[index]
  choose(example.kind)
  area('typed').value = json(example.message)
  edited('typed')
}

for (const [index, example] of examples.entries()) {
  const address = (example.message as { address: string }).address
  menu.append(new Option(`${example.kind === 'command' ? 'Command' : 'Reply'} ${address}`, String(index)))
}
menu.addEventListener('change', () => load(Number(menu.value)))
for (const radio of kinds) {
  radio.addEventListener('change', () => {
    choose(radio.value as Kind)
    edited('bytes')
  })
}
for (const pane of ['typed', 'raw', 'bytes'] as const) {
  area(pane).addEventListener('input', () => edited(pane))
}
load(0)
