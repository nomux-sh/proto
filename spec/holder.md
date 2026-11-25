# Holder protocol

**Status: IMPLEMENTED, version 1.1.** The holder protocol is defined and implemented in the `nomux` repository. Its specification is [`docs/holder.md`](https://github.com/nomux-sh/nomux/blob/main/docs/holder.md) there: messages, encoding, ring buffer layout, discovery files, lease and version rule. The code is the crate `nomux-holder-protocol`. This document gives the versioning rule and the questions that are still open.

The holder protocol is not a part of the `nomux-proto` crate. It has a small binary encoding of its own, because the holder must stay small and has no async runtime.

## Purpose

A session holder (`nomux-holder`) is a small process that owns the PTY of one session.
The daemon (`nomuxd`) connects to each holder on the same host.
The holder protocol is the protocol between one holder and the daemon.

The holder protocol is not the client protocol. Clients (CLI, desktop, mobile) speak to the daemon with the control protocol and terminal frames. Clients do not speak to a holder.

## Versioning rule

The holder protocol has the most strict versioning rule in nomux:

1. The daemon supports all holder protocol versions of the same major product version.
2. A holder that runs is never replaced and never upgraded. A holder keeps its protocol version until its session ends.
3. A new holder version is used only for new sessions.

The result: an upgrade of the daemon does not stop a session.

The effects on the design:

- The holder protocol has its own version number. This number is independent of the control protocol version in [compatibility.md](compatibility.md).
- A change to the holder protocol in a major version must be an addition. An old holder must continue to operate with a new daemon.
- A daemon can meet a holder that is much older than the daemon. Thus the daemon must read the holder protocol version before it sends other messages.
- Only a new major product version can remove an old holder protocol version.

## Decided

The specification in the `nomux` repository answers these questions:

- **Encoding.** A small binary encoding with length-prefixed messages, not protobuf.
- **Version exchange.** The holder sends its protocol version in the first message on the socket. A minor version adds messages. A holder answers an unknown request with a defined error.
- **Transport.** A Unix socket carries control messages. A ring buffer of fixed size in shared memory carries terminal output. The holder passes a read-only descriptor of the memory to the daemon.
- **Ring buffer layout.** A header with a claimed and a committed write position. A reader that is too slow detects that it was overrun.
- **Relation to the terminal frame header.** The ring buffer contains raw terminal output. The daemon adds the frame header from [frame.md](frame.md) when it sends data to clients.
- **Lease.** One connection at a time can write. The last request for the lease wins.
- **Discovery.** Each holder has a private directory with the files `pid` (process ID and process start time), `sock` and `ready`. The holder writes `ready` last.
- **Input, resize and signals.** One request for each.
- **Screen redraw.** The holder keeps the screen state and gives the byte sequence that reproduces the screen, together with the ring buffer position that the sequence corresponds to.

## Open questions

1. **Epoch and sequence number.** Which event starts a new epoch. The unit of the sequence number. [frame.md](frame.md) has the rule that the daemon uses now, without a journal: one epoch for the life of a Session, and the byte position of the ring buffer as the sequence number.
2. **Journal and checkpoint format.** The format on disk, and whether this format is a part of the holder protocol or private to the holder.
3. **Input line state.** The report of the prompt marks (OSC 133 B and C) and of the program status (OSC 7501).
4. **Warm pool.** The message that assigns a session to a holder from the warm pool.
5. **Compatibility tests.** How CI tests a new daemon with each older holder version of the same major version.

## Not in this document

- The control protocol between clients and the daemon. See the schemas in `proto/nomux/wire/v1/`.
- The terminal frame header. See [frame.md](frame.md).
