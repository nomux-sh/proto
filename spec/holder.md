# Holder protocol

**Status: OUTLINE.** Milestone M1 designs the holder protocol. This document gives the rules that are already decided and the open questions.

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

## Known design constraints

These constraints come from the architecture. They are inputs for the M1 design, not a specification.

- **Transport.** A Unix socket carries control messages. A ring buffer of fixed size in shared memory (`memfd`) carries terminal data. The daemon maps the buffer and reads it without a copy.
- **Discovery.** Each holder has a directory `~/.nomux/run/<id>/` with the files `pid`, `sock`, and `ready`. The `pid` file contains the process ID and the process start time. The holder writes `ready` last.
- **Single writer lease.** Only one daemon at a time can write to a holder. During a daemon upgrade, the new daemon takes the lease from the old daemon.
- **Journal and checkpoint.** The holder writes screen changes to an ordered journal and writes checkpoints at intervals. Each session has an epoch and a sequence number.
- **No runtime.** The holder has no async runtime. The protocol must be simple to implement with blocking I/O and `poll`.
- **State.** The holder keeps the smallest copy of the state of its session. When the daemon connects again, the daemon reads this state from the holder.

## Open questions for M1

1. **Encoding.** Does the control part use protobuf (and thus this repository's schemas), or a smaller fixed binary encoding? The holder must stay small and must be able to decode messages from all later daemon versions of the same major version.
2. **Version exchange.** Where is the holder protocol version stored: in the first message on the socket, in a discovery file, or in both? The daemon must know the version before it sends a message.
3. **Ring buffer layout.** The header of the shared memory region, the size, the read and write positions, and the rule for a reader that is too slow.
4. **Relation to the terminal frame header.** Does the ring buffer or the journal contain the frame header from [frame.md](frame.md), or does the daemon add the header when it sends data to clients?
5. **Lease.** The messages to take, renew, and release the single writer lease, and the behavior when two daemons request the lease at the same time.
6. **Epoch and sequence number.** Which event starts a new epoch. The unit of the sequence number. The same question is open in [frame.md](frame.md).
7. **Journal and checkpoint format.** The format on disk, and whether this format is a part of the holder protocol or private to the holder.
8. **Input and resize.** The messages for input bytes, window size, and signals, and the report of the input line state (OSC 133 B and C).
9. **Warm pool.** The message that assigns a session to a holder from the warm pool.
10. **Compatibility tests.** How CI tests a new daemon with each older holder version of the same major version.

## Not in this document

- The control protocol between clients and the daemon. See the schemas in `proto/nomux/wire/v1/`.
- The terminal frame header. See [frame.md](frame.md).
