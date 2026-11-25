# Terminal frames

**Status: DRAFT.** The layout and the rules can change in a minor version until version 1.0.0. [Open items](#open-items) lists what is not decided.

This document specifies the binary frames that carry terminal data: the fixed header, the payload of each frame kind, and the terminal data path between the daemon and a client that is attached to a Session.
The implementation of the header and of the `End` payload is the module `frame` of the crate `nomux-proto`.

The daemon has the output of a Session in two places: the ring buffer of the holder has the last bytes, and the journal on disk has the bytes before them. The holder writes the two. [`docs/holder.md`](https://github.com/nomux-sh/nomux/blob/main/docs/holder.md) of the `nomux` repository specifies them.

## Scope

Terminal data does not use protobuf. Terminal data moves in binary frames on the terminal data channel and on the input channel.

A frame is a header and then a payload:

```
+----------------------+---------------------------+
| header (24 bytes)    | payload (`length` bytes)  |
+----------------------+---------------------------+
```

The control messages that start and end an attach (`Session.Attach`, `Session.Detach`, `Session.Input`, `Session.Resize`) are in [control.md](control.md).

## Layout

The header has a fixed length of 24 bytes.
All integers are unsigned and little-endian.

| Offset | Size | Field     | Type  | Description |
|-------:|-----:|-----------|-------|-------------|
| 0      | 1    | `version` | `u8`  | The version of the header layout. The value is `1`. |
| 1      | 1    | `kind`    | `u8`  | The frame kind. See [Frame kinds](#frame-kinds). |
| 2      | 2    | `flags`   | `u16` | Reserved. The value is `0`. |
| 4      | 4    | `session` | `u32` | The session handle. |
| 8      | 4    | `epoch`   | `u32` | The epoch of the session. |
| 12     | 4    | `length`  | `u32` | The number of payload bytes after the header. |
| 16     | 8    | `seq`     | `u64` | The sequence number in the epoch. |

Byte map:

```
 0        1        2        3        4        5        6        7
+--------+--------+--------+--------+--------+--------+--------+--------+
|version | kind   |      flags      |              session              |
+--------+--------+--------+--------+--------+--------+--------+--------+
 8        9        10       11       12       13       14       15
+--------+--------+--------+--------+--------+--------+--------+--------+
|               epoch               |              length               |
+--------+--------+--------+--------+--------+--------+--------+--------+
 16       17       18       19       20       21       22       23
+--------+--------+--------+--------+--------+--------+--------+--------+
|                                  seq                                  |
+--------+--------+--------+--------+--------+--------+--------+--------+
```

Each field starts at an offset that is a multiple of its size. The header has no padding bytes.

### Example

A header with `kind` = `Output`, `session` = `0x01020304`, `epoch` = `0x05060708`, `length` = `0x000a0b0c`, and `seq` = `0x1112131415161718` has these bytes:

```
01 01 00 00  04 03 02 01  08 07 06 05  0c 0b 0a 00
18 17 16 15  14 13 12 11
```

The unit tests of the crate use this example.

## Fields

### `version`

The version of the header layout. This document specifies version `1`.

If the layout changes in a manner that is not compatible, the version increases.
The `version` field stays at offset 0 in all versions.

### `kind`

The frame kind. The value `0` is not a valid kind.

### `flags`

Reserved for future use, for example to mark a compressed payload.
A sender sets the field to `0`.
A capability flag from [capabilities.md](capabilities.md) must be agreed in the handshake before a peer sends a flag bit.

### `session`

A handle that identifies one attach on one connection.
The handle is not the permanent identifier of the Session.
The daemon selects the handle and gives it in `SessionAttachResponse.handle`. The value is not `0`.
Each frame of an attach, in the two directions, has this handle.

### `epoch`

The epoch of the bytes of the frame. One epoch is one continuous numbering of the output of a Session.
See [Epoch and sequence number](#epoch-and-sequence-number).

### `length`

The number of payload bytes after the header.
The largest permitted value is `1048576` (1 MiB). A sender must split larger data into more than one frame.

A `length` of `0` is valid.

### `seq`

The sequence number in the epoch. The unit is one byte of terminal output.
See [Epoch and sequence number](#epoch-and-sequence-number).

## Frame kinds

| Value | Name             | Direction | Description |
|------:|------------------|-----------|-------------|
| 0     | (not valid)      |           | Never sent. |
| 1     | `Output`         | daemon to client | Terminal output bytes of a Session. |
| 2     | `ScreenSnapshot` | daemon to client | A full copy of the screen. |
| 3     | `ScreenDiff`     | daemon to client | The rows that changed on the screen. Not sent in this version. |
| 4     | `Ack`            | client to daemon | The receiver confirms the frames up to a sequence number. Not sent in this version. |
| 5     | `Credit`         | client to daemon | The receiver gives flow-control credit to the sender. Not sent in this version. |
| 6     | `Input`          | client to daemon | Input bytes for a Session. Used on the input channel. |
| 7     | `End`            | daemon to client | The attach ended. It is the last frame of an attach. |
| 8     | `HistoryGap`     | daemon to client | The daemon does not have a part of the output that the client asked for. Capability flag `output_history`. |
| 9     | `Restored`       | daemon to client | The command of the Session started again in a new epoch. Capability flag `output_history`. |

Values 10 to 255 are not assigned.
The kinds 1 to 7 are the base set of control protocol version 1. A kind with a larger value needs a capability flag: a peer must not send it unless the other peer advertised the related capability flag in the handshake. [capabilities.md](capabilities.md) is the registry of the flags.

## Payloads

### `Output`

The payload is bytes that the command of the Session wrote to its terminal, without a change.
`seq` is the sequence number of the first byte of the payload.

A frame can end in the middle of an escape sequence or of a UTF-8 character. The next `Output` frame continues it.

### `ScreenSnapshot`

The payload is bytes of a redraw sequence. A redraw sequence is terminal output (text and escape sequences) that makes the current screen of the Session in a terminal that is in its initial state and has the window size of the Session: the visible cells with their attributes, the cursor, and the terminal modes. If the alternate screen is visible, the sequence also makes the screen behind it.

- One snapshot is one or more `ScreenSnapshot` frames that follow each other. Each frame of one snapshot has the same `seq`. The payloads, in their sequence, are the redraw sequence.
- `seq` is the sequence number that the screen corresponds to: the snapshot has the effect of each output byte before `seq`, and of no byte from `seq` on.
- Two snapshots of one attach never have the same `seq`. Thus a `ScreenSnapshot` frame starts a new snapshot if it is the first frame of the attach, if the frame before it has a different kind, or if the frame before it has a different `seq`.
- Before a client applies the first byte of a snapshot, it puts its terminal into the initial state. The redraw sequence does not make the scrollback.

### `Input`

The payload is bytes for the terminal of the Session, as the keyboard of the user makes them.
The sender sets `epoch` and `seq` to `0`, and the receiver ignores the two fields.

### `End`

The payload has 8 bytes. All integers are little-endian.

| Offset | Size | Field    | Type  | Description |
|-------:|-----:|----------|-------|-------------|
| 0      | 1    | `reason` | `u8`  | Why the attach ended. |
| 1      | 3    |          |       | The sender writes `0`, and the receiver ignores the value. |
| 4      | 4    | `value`  | `i32` | The exit code or the signal number. `0` for the other reasons. |

| `reason` | Name       | Meaning | `value` |
|---------:|------------|---------|---------|
| 1        | `Exited`   | The command of the Session ended with an exit code. | The exit code. |
| 2        | `Signaled` | A signal ended the command of the Session. | The signal number of the operating system of the Host. |
| 3        | `Detached` | A `Session.Detach` request ended the attach. The Session continues. | `0` |
| 4        | `Lost`     | The Session ended, and the daemon does not know how the command ended. | `0` |

`seq` is the sequence number after the last output byte that the daemon sent.
A receiver ignores the bytes after the fields that it knows, and it treats a `reason` that it does not know as an end without more data.

### `HistoryGap`

The payload is empty. A receiver ignores the payload bytes of a later version.

The frame tells the client that output is gone: the daemon does not have the output bytes between the last byte that the client has and the next frame. The client cannot get these bytes from the daemon.

- `epoch` is the epoch of the bytes that are gone, and `seq` is the sequence number of the first byte that is gone.
- The next frame starts a snapshot of the same epoch, or it is `Restored`.

A snapshot without a `HistoryGap` before it also skips output: see [Slow client](#slow-client). The daemon still has that output, and the client can read it with `Session.Capture`.

### `Restored`

The payload is empty. A receiver ignores the payload bytes of a later version.

The frame tells the client that the command of the Session started again after the Session was dormant.

- `epoch` is the new epoch, and `seq` is `0`.
- Each frame before this frame is of an earlier epoch: it is output from before the restore. Each frame after it is of the new epoch.
- The next frame starts a snapshot.

A client shows the user a mark at this place: the output before it is from the command that stopped, and the processes of that command do not run.

### `ScreenDiff`, `Ack` and `Credit`

The payloads of these kinds are not defined. A peer does not send them in this version.

## Decode rules

A receiver does these checks in this order. The receiver stops at the first check that fails.

1. If fewer than 24 bytes are available, wait for more bytes.
2. If `version` is not `1`, the header is not valid. The receiver must not read the other fields, because their layout is not known. The receiver closes the channel.
3. If `flags` is not `0`, the header is not valid. The receiver closes the channel.
4. If `length` is larger than `1048576`, the header is not valid. The receiver closes the channel.
5. If `kind` is not a known value, the frame is not valid. The `length` field is correct, thus the receiver can skip the payload. To send a kind that was not agreed in the handshake is a protocol error.
6. If `session` is not the handle of an attach of the connection, the receiver ignores the frame.

## Terminal data path

### Channels

An attach uses two channels. Each channel is a reliable, ordered byte stream in one direction.

| Channel | Direction | Frames |
|---------|-----------|--------|
| Terminal data channel | daemon to client | `ScreenSnapshot`, `Output`, `HistoryGap`, `Restored`, `End` |
| Input channel | client to daemon | `Input` |

The two channels are independent: a channel that is full does not stop the other channel.
[Transports](#transports) says how a transport gives the two channels.

### Start of an attach

1. The client sends `SessionAttachRequest` on the control channel.
2. If the Session is dormant, the daemon starts its command again. See [Restore](#restore).
3. If the request has a window size, the daemon gives that size to the Session. [control.md](control.md) has the rule for the window size.
4. The daemon sends `SessionAttachResponse` with the handle and the current epoch.
5. The daemon sends the screen: one snapshot. If the request asks to resume and the daemon can do it, the daemon sends no snapshot. See [Resume](#resume).
6. The daemon sends each output byte from the `seq` of the snapshot in `Output` frames, as the command writes it. The daemon sends output without a request from the client.

After a snapshot with the sequence number `S`, the first `Output` frame has the `seq` `S`. An `Output` frame with the `seq` `N` and the `length` `L` is followed by an `Output` frame with the `seq` `N + L`, or by a new snapshot.
Thus a client gets each output byte one time: there is no gap and no repeat between a snapshot and the output after it.

### Epoch and sequence number

- The sequence number of an output byte is the number of bytes that the command of the Session wrote to its terminal before that byte, in the current epoch. The first byte of an epoch has the sequence number `0`.
- An epoch is one continuous numbering. In one epoch, a sequence number always means the same byte.
- A Session gets a new, larger epoch only when the numbering starts again at `0`. A client must then discard the sequence number that it has.
- The value `0` is not an epoch. It means "no epoch".

The holder of a Session counts the bytes and gives the epoch. Thus the sequence numbers continue when the daemon stops and a new daemon starts.

- The first epoch of a Session is `1` (`nomux_proto::frame::FIRST_EPOCH`).
- A Session gets the next epoch when a new holder starts for it: a restore of a dormant Session. That is the only event that starts the numbering again.
- Each frame has the epoch of its bytes in the `epoch` field. `SessionAttachResponse.epoch` is the current epoch of the Session. A frame of an earlier epoch comes only before a `Restored` frame.

### Resume

A client that had an attach to the Session can ask for only the bytes that it does not have. It sends in `SessionAttachRequest` the `epoch` of its last attach and, as `from_seq`, the sequence number after the last output byte that it applied.

The daemon resumes if these conditions are true:

- `epoch` is the current epoch of the Session.
- `from_seq` is not larger than the sequence number of the next output byte.
- The daemon still has each output byte from `from_seq` on, in the ring buffer or in the journal.

Then the daemon sends no snapshot. The first `Output` frame has the `seq` `from_seq`. The daemon sends the bytes of the journal first and continues with the bytes of the ring buffer. The client sees no difference: each `Output` frame continues the frame before it.

If a condition is false, the daemon sends a snapshot, as for a client that sends the epoch `0`. The client learns the result from the kind of the first frame.

A client can resume only if it kept its terminal state from the earlier attach. A client that starts with a new terminal sends the epoch `0`.
A resume works after a restart of the daemon, because the holder keeps the ring buffer, the journal and the sequence numbers.

#### Output that the journal does not have

The journal of a Session has a size limit, and the holder removes the oldest output. A file of the journal can also be damaged. Thus the daemon can have the newest output of an epoch and not the byte at `from_seq`.

| Capability flag `output_history` | What the daemon sends |
|----------------------------------|-----------------------|
| Active | `HistoryGap` with the `seq` `from_seq`. Then the oldest state that the journal has after the gap: one snapshot with the `seq` `S` of the oldest checkpoint, and each output byte from `S` on, first from the journal and then from the ring buffer. If the journal has no checkpoint, the snapshot is the current screen. |
| Not active | One snapshot of the current screen, and the output from its `seq` on. |

In the two cases the client gets no byte between `from_seq` and the `seq` of the snapshot.

#### An earlier epoch

A client that was attached before a restore has a position in an earlier epoch. If the capability flag `output_history` is active and `epoch` is smaller than the current epoch, the daemon sends these frames:

1. The output of the epoch `epoch` from `from_seq` to the end of that epoch, in `Output` frames that have that epoch. The output stops at the first byte that the journal does not have: the daemon then sends `HistoryGap` and no more output of that epoch.
2. `Restored` with the current epoch.
3. One snapshot of the current epoch, and the output from its `seq` on.

The daemon does not send the output of an epoch between the two. Without the capability flag, the daemon sends only the snapshot.

### Restore

A dormant Session has no process: its holder is gone, and its command did not end. [control.md](control.md) has the states of a Session. An attach to a dormant Session starts the command again, in the next epoch.

If the capability flag `output_history` is active and the request has the epoch `0`, the client gets the end of the earlier epoch before the new screen:

1. One snapshot of the earlier epoch: the last checkpoint that the journal has for it.
2. The output of the earlier epoch from the `seq` of that snapshot to its end.
3. `Restored` with the new epoch.
4. One snapshot of the new epoch, and the output from its `seq` on.

If the journal has no checkpoint of the earlier epoch, the daemon starts at step 3. The frames of steps 1 and 2 have the earlier epoch in the `epoch` field.
A client that sends an earlier epoch gets the frames of [An earlier epoch](#an-earlier-epoch).

The checkpoints of a journal are 1 MiB of output apart. Thus steps 1 and 2 give the last screen of the earlier epoch and 1 MiB of its output at most. A client reads the output before that with `Session.Capture`.

Only the attach that starts the command again gets these frames without a request. A later attach with the epoch `0` gets the snapshot of the current epoch.

### Slow client

The daemon keeps no queue of output for a client.

- The daemon reads the output for a client only when the terminal data channel of that client can take more bytes. It holds one frame of output for each attach at most.
- The holder never waits for a client or for the daemon. It writes the output into a ring buffer of a fixed size and overwrites the oldest bytes.
- If the bytes that a client needs next are no longer in the ring buffer, the client is behind. The daemon then sends a new snapshot and continues with `Output` frames from the `seq` of that snapshot. This `seq` is larger than the sequence number that the client expected: the client did not get the bytes between the two, and the snapshot has their effect. The daemon sends no `HistoryGap`: the journal has these bytes, and the client can read them with `Session.Capture`.

Thus a client that is slow gets the latest screen and not each byte, a slow client does not make the daemon use more memory, and a client that keeps up gets each byte.
Each attach reads the ring buffer independently: a slow client changes nothing for a different client of the same Session.

The transport gives the flow control of a channel: the send buffer of a Unix socket, the window of an SSH channel or of a QUIC stream. The `Credit` frame is for a transport without flow control, and no transport of this version needs it.

### Input

- An attached client sends each key and each pasted text in `Input` frames on the input channel. The daemon sends no reply.
- The input channel has priority: the daemon reads the input channel of a client also while the terminal data channel of that client is full, and before it sends more output to that client. Input never waits for output.
- A client sends an `Input` frame immediately. It does not collect input to make larger frames.
- An `Input` frame for a Session whose command has ended is ignored.

The window size does not go on the input channel. A client sends `Session.Resize` on a control channel.

### End of an attach

An attach ends in one of these manners:

| Event | What the client receives |
|-------|--------------------------|
| The command of the Session ends | The output that remains, then `End` with `Exited` or `Signaled`. The daemon closes the terminal data channel. |
| A `Session.Detach` request names the Session | `End` with `Detached`. The daemon closes the terminal data channel. |
| The holder of the Session is gone and the daemon cannot get the exit status | `End` with `Lost`. If the command ran, the Session is dormant then. |
| The client closes the channels | Nothing. This is how a client detaches itself. |
| The daemon stops or fails | The channels close without an `End` frame. The Session continues, and the client can attach again. |

The end of an attach never changes the Session or a different attach.

### Transports

| Transport | Channels of an attach |
|-----------|-----------------------|
| Unix socket (local Host) | One connection for each attach. The client connects, does the handshake and sends `SessionAttachRequest`. After `SessionAttachResponse`, the connection carries only frames: the direction from the daemon is the terminal data channel, and the direction from the client is the input channel. The client sends no frame before it has the response. To send a control request during an attach, the client uses a second connection. |
| SSH, QUIC | Not defined yet. Each channel will be a separate SSH channel or QUIC stream, and the control channel stays open. |

On the Unix socket, the handle of an attach is always `1`, because a connection has one attach.

## Design decisions

- **No magic number.** A frame is sent on a dedicated, reliable, ordered channel (an SSH channel or a QUIC stream) after the handshake. A magic number in each frame does not help a receiver to find the next frame on such a channel, and it adds bytes to each small interactive frame. The `version` byte detects a peer that uses a different layout.
- **Version in each header.** One byte in each header lets a receiver reject a layout that it does not know before it reads the other fields. The byte does not make the header longer, because the fields before `session` fill 4 bytes for alignment.
- **Little-endian.** All supported targets (x86_64 and aarch64) are little-endian. A host can write and read the fields without a byte swap.
- **Fixed length, no optional fields.** A receiver reads exactly 24 bytes and then knows the payload length. The payload is not copied to decode the header.
- **No `zerocopy` dependency.** The codec reads six integers with the standard library functions `from_le_bytes` and `to_le_bytes`. The code has no `unsafe`. A dependency does not make this code simpler or faster.
- **Handle for `session`.** The permanent identifier of a session is longer than 4 bytes. A handle for each connection keeps the header small.

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The payloads of `ScreenDiff`, `Ack` and `Credit`, and a second send mode in which a client that is behind gets only the rows that changed.
- The output of the epochs between the epoch of a client and the current epoch.
- A message that tells the attached clients a new window size of the Session.
- The channels of an attach on SSH and on QUIC, and more than one attach on one connection.
- The size of `session`, `epoch`, and `seq`. For example, a larger `session` field for a permanent identifier.
- The payload limit of 1 MiB, and a smaller limit for interactive channels.
- The use of `flags` for zstd compression.
- A fuzz target for the frame codec.
