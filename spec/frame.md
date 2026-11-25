# Terminal frame header

**Status: DRAFT.** The layout can change in a minor version until version 1.0.0. [Open items](#open-items) lists what is not decided.

This document specifies the fixed binary header of a terminal data frame.
The implementation is the module `frame` of the crate `nomux-proto`.

## Scope

Terminal data does not use protobuf. Terminal data moves in binary frames on the terminal data channel and on the input channel.

A frame is a header and then a payload:

```
+----------------------+---------------------------+
| header (24 bytes)    | payload (`length` bytes)  |
+----------------------+---------------------------+
```

This document specifies the header.
This document does not specify the payload of each frame kind. See [Open items](#open-items).

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

A handle that identifies one session on one connection.
The handle is not the permanent identifier of the session.
The control protocol will assign the handle when a client attaches to a session. This part is not defined yet.

### `epoch`

The epoch of the session. A session has an epoch and a sequence number. A client that connects again gives the epoch and the last sequence number that it received.

### `length`

The number of payload bytes after the header.
The largest permitted value is `1048576` (1 MiB). A sender must split larger data into more than one frame.

A `length` of `0` is valid.

### `seq`

The sequence number in the epoch.
The unit of the sequence number (frames or bytes) is not defined yet. See [Open items](#open-items).

## Frame kinds

| Value | Name             | Description |
|------:|------------------|-------------|
| 0     | (not valid)      | Never sent. |
| 1     | `Output`         | Terminal output bytes of a session. |
| 2     | `ScreenSnapshot` | A full copy of the screen. |
| 3     | `ScreenDiff`     | The rows that changed on the screen. |
| 4     | `Ack`            | The receiver confirms the frames up to a sequence number. |
| 5     | `Credit`         | The receiver gives flow-control credit to the sender. |
| 6     | `Input`          | Input bytes for a session. Used on the input channel. |

Values 7 to 255 are not assigned.
A new kind needs a capability flag. A peer must not send a kind unless the other peer advertised the related capability flag in the handshake.

## Decode rules

A receiver does these checks in this order. The receiver stops at the first check that fails.

1. If fewer than 24 bytes are available, wait for more bytes.
2. If `version` is not `1`, the header is not valid. The receiver must not read the other fields, because their layout is not known. The receiver closes the channel.
3. If `flags` is not `0`, the header is not valid. The receiver closes the channel.
4. If `length` is larger than `1048576`, the header is not valid. The receiver closes the channel.
5. If `kind` is not a known value, the frame is not valid. The `length` field is correct, thus the receiver can skip the payload. To send a kind that was not agreed in the handshake is a protocol error.

## Design decisions

- **No magic number.** A frame is sent on a dedicated, reliable, ordered channel (an SSH channel or a QUIC stream) after the handshake. A magic number in each frame does not help a receiver to find the next frame on such a channel, and it adds bytes to each small interactive frame. The `version` byte detects a peer that uses a different layout.
- **Version in each header.** One byte in each header lets a receiver reject a layout that it does not know before it reads the other fields. The byte does not make the header longer, because the fields before `session` fill 4 bytes for alignment.
- **Little-endian.** All supported targets (x86_64 and aarch64) are little-endian. A host can write and read the fields without a byte swap.
- **Fixed length, no optional fields.** A receiver reads exactly 24 bytes and then knows the payload length. The payload is not copied to decode the header.
- **No `zerocopy` dependency.** The codec reads six integers with the standard library functions `from_le_bytes` and `to_le_bytes`. The code has no `unsafe`. A dependency does not make this code simpler or faster.
- **Handle for `session`.** The permanent identifier of a session is longer than 4 bytes. A handle for each connection keeps the header small.

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The payload layout of each frame kind.
- The values of `session`, `epoch`, and `seq` for kinds that do not belong to one session, for example `Credit`.
- The unit of `seq`: one for each frame, or the byte offset in the output.
- The size of `session`, `epoch`, and `seq`. For example, a larger `session` field for a permanent identifier.
- The payload limit of 1 MiB, and a smaller limit for interactive channels.
- The use of `flags` for zstd compression.
- A fuzz target for the frame codec.
