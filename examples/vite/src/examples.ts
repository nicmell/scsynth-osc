/** Whether a typed form is a command, which a client sends to scsynth, or
 * a reply, which scsynth sends back. */
export type Kind = 'command' | 'reply'

export interface Example {
  kind: Kind
  /** The typed form: the serde form of the library's Command or Reply. */
  message: unknown
}

export const examples: Example[] = [
  {
    kind: 'command',
    message: {
      address: '/s_new',
      args: {
        def: 'default',
        id: -1,
        action: 'head',
        target: 1,
        controls: [
          { control: 'freq', value: 440 },
          { control: 'amp', value: 0.25 },
          { control: 'partials', value: [1, 0.5, 'c4'] },
        ],
      },
    },
  },
  {
    kind: 'command',
    message: {
      address: '/n_set',
      args: {
        node: 1000,
        controls: [
          { control: 0, value: 330.5 },
          { control: 'amp', value: 'c3' },
        ],
      },
    },
  },
  {
    kind: 'command',
    message: {
      address: '/b_alloc',
      args: {
        buffer: 0,
        frames: 44100,
        channels: 2,
        completion: { message: { address: '/b_query', args: { buffers: [0] } } },
        sample_rate: 0,
      },
    },
  },
  {
    kind: 'command',
    message: {
      address: '/d_recv',
      args: { data: [83, 67, 103, 102, 0, 0, 0, 2], completion: null },
    },
  },
  { kind: 'command', message: { address: '/sync', args: 7 } },
  {
    kind: 'reply',
    message: {
      address: '/status.reply',
      args: {
        ugens: 12,
        synths: 3,
        groups: 2,
        synthdefs: 40,
        avg_cpu: 1.5,
        peak_cpu: 3.25,
        nominal_sample_rate: 48000,
        actual_sample_rate: 47999.5,
      },
    },
  },
  {
    kind: 'reply',
    message: {
      address: '/n_go',
      args: { node: 1000, parent: 1, prev: null, next: null, kind: 'synth' },
    },
  },
  {
    kind: 'reply',
    message: { address: '/done', args: { buffer: { command: '/b_alloc', buffer: 0 } } },
  },
  {
    kind: 'reply',
    message: {
      address: '/fail',
      args: { command: '/s_new', message: 'duplicate node ID', value: null },
    },
  },
  { kind: 'reply', message: { address: '/tr', args: { node: 1000, id: 7, value: 0.5 } } },
]
