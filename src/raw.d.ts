/** One OSC message: its address and its arguments. */
export interface RawMessage {
  address: string;
  args: RawArg[];
}

/** An argument of a RawMessage: its OSC tag as the type, and its data. An array is a bare array of arguments. */
export type RawArg =
  | { type: "i" | "f" | "d"; data: number }
  | { type: "h"; data: number | bigint }
  | { type: "s" | "c"; data: string }
  | { type: "b"; data: Uint8Array }
  | { type: "t"; data: { seconds: number; fraction: number } }
  | { type: "r"; data: { red: number; green: number; blue: number; alpha: number } }
  | { type: "m"; data: { port: number; status: number; data1: number; data2: number } }
  | { type: "T" | "F" | "N" | "I" }
  | RawArg[];
