# Control channel

**Status: DRAFT.** The rules can change in a minor version until version 1.0.0.

This document specifies how control messages move on the control channel, and it lists the services and their messages.
The schemas are in `proto/nomux/wire/v1/`. The schemas are the authority for the fields of each message.

## Framing

The control channel is a byte stream. Each message on the channel is one `Envelope`.

1. The sender writes the length of the encoded `Envelope` as a protobuf varint.
2. The sender writes the encoded `Envelope`.

This is the length-delimited encoding of protobuf.

The maximum length of one encoded `Envelope` is 1048576 bytes (1 MiB). The constant `nomux_proto::wire::v1::MAX_ENVELOPE_LENGTH` has this value.
If a peer reads a length that is larger than the maximum, the peer closes the connection.

## Requests and responses

- The client sends a `Request` with an `id`. The `id` is not 0. It is different from the `id` of each request on the same connection that has no response yet.
- The daemon sends one `Response` with the same `id`. The response has a result, or it has an `Error`.
- The first request on a connection is `Hello`. [compatibility.md](compatibility.md) has the version rule.
- If the daemon does not know the body of a request, it sends an `Error` with the code `ERROR_CODE_FAILED`. The connection stays open.

## Error codes

Each value of `ErrorCode` is equal to the related exit code of the `nomux` command-line client.

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `ERROR_CODE_UNSPECIFIED` | The daemon did not set the field. The client treats this as value 1. |
| 1 | `ERROR_CODE_FAILED` | The request failed, and no other code gives the reason. |
| 2 | `ERROR_CODE_NOT_FOUND` | The item that the request names does not exist. |
| 3 | `ERROR_CODE_TIMEOUT` | The daemon did not complete the request in the permitted time. |
| 4 | `ERROR_CODE_PERMISSION_DENIED` | The client does not have permission. |
| 5 | `ERROR_CODE_VERSION_INCOMPATIBLE` | The client and the daemon have no common control protocol version. |

The exit code 0 (success) has no error code.

## Services

Each service has its own block of 100 field numbers in `Request.body` and in `Response.result`. A request and its response use the same field number.

| Service | Field numbers | Schema |
|---------|---------------|--------|
| Handshake | 100 to 199 | `handshake.proto` |
| Host | 200 to 299 | `host.proto` |
| Session | 300 to 399 | `session.proto` |

### Handshake

| Field number | Request | Response |
|-------------:|---------|----------|
| 100 | `Hello` | `Welcome` |

### Host

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 200 | `Host.Info` | `HostInfoRequest` | `HostInfoResponse` | Gives the product version of the daemon, its process ID and the number of Sessions. |
| 201 | `Host.Stop` | `HostStopRequest` | `HostStopResponse` | Stops the daemon. The daemon sends the response, closes each connection and exits. No Session ends. |

### Session

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 300 | `Session.Create` | `SessionCreateRequest` | `SessionCreateResponse` | Starts a new Session and gives its identifier, its state and the process ID of its command. |
| 301 | `Session.List` | `SessionListRequest` | `SessionListResponse` | Gives all Sessions on the Host. |
| 302 | `Session.Kill` | `SessionKillRequest` | `SessionKillResponse` | Ends a Session. The daemon answers after the command of the Session has ended. |

Errors of the Session service:

| Request | Condition | Code |
|---------|-----------|------|
| `SessionCreateRequest` | The working directory is not an absolute path of a directory, or the command cannot start. | `ERROR_CODE_FAILED` |
| `SessionCreateRequest` | The Session was not ready in the permitted time. | `ERROR_CODE_TIMEOUT` |
| `SessionKillRequest` | No Session has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `SessionKillRequest` | The command did not end in the permitted time. | `ERROR_CODE_TIMEOUT` |

A Session has one of these states:

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `SESSION_STATE_UNSPECIFIED` | The daemon did not set the field. The client treats the state as unknown. |
| 1 | `SESSION_STATE_RUNNING` | The command of the Session runs. |
| 2 | `SESSION_STATE_UNREACHABLE` | The holder of the Session runs, but the daemon has no connection to it. The daemon tries to connect again. |

When the command of a Session ends, the daemon removes the Session from its list.

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The messages to attach to a Session, to send input and to change the window size.
- The environment and the window size of a new Session.
- A state for a Session whose command has ended, with the exit status.
- The other services: Project, Worktree, Agent, Team, Task, Events and Ext.
