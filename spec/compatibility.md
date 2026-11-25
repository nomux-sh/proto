# Compatibility

**Status: DRAFT.** The rules can change in a minor version until version 1.0.0.

This document specifies how a client and a daemon agree on a control protocol version.

## Control protocol version

The control protocol version is an integer. The first version is `1`. The value `0` is not a valid version.

The version increases by 1 when a change to the control protocol or to the terminal frames is not compatible with the previous version.
A change that only adds an optional feature does not increase the version. Such a change adds a capability flag. See [capabilities.md](capabilities.md).

Until the product release `1.0.0`, version `1` is not stable. Development builds with version `1` can be incompatible with each other.

## Handshake

Each peer sends two values:

| Field     | Meaning |
|-----------|---------|
| `version` | The newest control protocol version that the peer can use. |
| `oldest`  | The oldest control protocol version that the peer can use. |

The client sends the values in `Hello`. The daemon sends the values in `Welcome`.

A range is valid if `oldest` is not `0` and `oldest` is not larger than `version`.

## Version rule

Let `C` be the range of the client and `D` be the range of the daemon.

1. The selected version is the smaller of `C.version` and `D.version`.
2. If the selected version is smaller than `C.oldest` or smaller than `D.oldest`, the peers are not compatible.
3. If the peers are compatible, the two peers use the selected version for the connection.

Examples:

| Client (`oldest` to `version`) | Daemon (`oldest` to `version`) | Result |
|--------------------------------|--------------------------------|--------|
| 1 to 3                         | 1 to 3                         | Version 3 |
| 1 to 3                         | 2 to 5                         | Version 3 |
| 3 to 5                         | 1 to 3                         | Version 3 |
| 1 to 3                         | 4 to 5                         | Not compatible |
| 4 to 5                         | 1 to 3                         | Not compatible |

The function `nomux_proto::compatibility::negotiate` implements this rule.

## When the peers are not compatible

If the daemon finds that the peers are not compatible, the daemon does not send `Welcome`.
The daemon sends a `Response` with an `Error` that has the code `ERROR_CODE_VERSION_INCOMPATIBLE` and a clear message. Then the daemon closes the connection.

If the client finds that the peers are not compatible, the client closes the connection.

In the two cases, the `nomux` command-line client prints the message and exits with **exit code 5** (version incompatible).

The value of `ERROR_CODE_VERSION_INCOMPATIBLE` is `5`, the same as the exit code.

## Support window

The daemon supports the clients of the last two minor product releases.
Thus the `oldest` value of a daemon is not larger than the control protocol version of the release that is two minor releases before the daemon.

An older client receives the error with code 5.

## Other protocols

- **Holder protocol.** The protocol between a holder and the daemon has its own version and a more strict rule. See [holder.md](holder.md).
- **Terminal frame header.** The header has a layout version in each frame. See [frame.md](frame.md).

## Compatibility table

The table gives the control protocol version of each product release. No release exists yet.

| Product release | Control protocol `version` | `oldest` |
|-----------------|----------------------------|----------|
| (not released)  | 1                          | 1        |

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- Whether the `Error` for an incompatible version also carries the `version` and `oldest` values of the daemon as fields, so that a client can print them without the message text.
- The framing of an `Envelope` on the control channel. The current draft is a varint length prefix before each `Envelope` (protobuf length-delimited encoding).
- The maximum size of an `Envelope`.
