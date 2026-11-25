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
| 303 | `Session.Attach` | `SessionAttachRequest` | `SessionAttachResponse` | Connects the client to the terminal of a Session. After the response, the client gets the screen and the output in terminal frames. |
| 304 | `Session.Detach` | `SessionDetachRequest` | `SessionDetachResponse` | Ends each attach to a Session and gives the number of clients that were attached. The Session continues. |
| 305 | `Session.Input` | `SessionInputRequest` | `SessionInputResponse` | Writes bytes to the terminal of a Session. |
| 306 | `Session.Resize` | `SessionResizeRequest` | `SessionResizeResponse` | Changes the window size of a Session. |

Errors of the Session service:

| Request | Condition | Code |
|---------|-----------|------|
| `SessionCreateRequest` | The working directory is not an absolute path of a directory, or the command cannot start. | `ERROR_CODE_FAILED` |
| `SessionCreateRequest` | The Session was not ready in the permitted time. | `ERROR_CODE_TIMEOUT` |
| `SessionKillRequest` | No Session has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `SessionKillRequest` | The command did not end in the permitted time. | `ERROR_CODE_TIMEOUT` |
| `SessionAttachRequest`, `SessionDetachRequest`, `SessionInputRequest`, `SessionResizeRequest` | No Session has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `SessionAttachRequest`, `SessionInputRequest`, `SessionResizeRequest` | The daemon has no connection to the holder of the Session (state `SESSION_STATE_UNREACHABLE`). | `ERROR_CODE_FAILED` |
| `SessionAttachRequest`, `SessionResizeRequest` | The window size is not valid. | `ERROR_CODE_FAILED` |
| `SessionInputRequest` | The command of the Session has ended. | `ERROR_CODE_FAILED` |

A Session has one of these states:

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `SESSION_STATE_UNSPECIFIED` | The daemon did not set the field. The client treats the state as unknown. |
| 1 | `SESSION_STATE_RUNNING` | The command of the Session runs. |
| 2 | `SESSION_STATE_UNREACHABLE` | The holder of the Session runs, but the daemon has no connection to it. The daemon tries to connect again. |

When the command of a Session ends, the daemon removes the Session from its list.

### Attach

`Session.Attach` starts the terminal data path of [frame.md](frame.md). That document specifies the frames, the sequence numbers, the resume, the rule for a slow client and the end of an attach.

- `SessionAttachResponse` is the last control message on its connection, in the two directions. After it, the connection carries terminal frames. To send a control request during an attach, a client uses a second connection.
- More than one client can be attached to one Session at the same time. Each client gets the same output.
- A client detaches itself when it closes the connection of its attach. No request is necessary.
- `Session.Detach` detaches all the clients of a Session, for example from a different terminal. Each of these clients gets an `End` frame with the reason `Detached`. A `Session.Detach` for a Session without an attached client succeeds with the number 0.
- When the command of a Session ends, each attached client gets the output that remains and then an `End` frame with the exit status.

### Window size

A Session has one window size. A window size is valid if `rows` and `columns` are from 1 to 65535 and the window has 1048576 cells at most.

- The window size of a Session is the size of the last request that gave one: a `SessionAttachRequest` in which `rows` and `columns` are not 0, or a `SessionResizeRequest`.
- A `SessionAttachRequest` in which `rows` and `columns` are 0 does not change the window size. A request in which only one of the two is 0 is not valid.
- A client that is attached sends `Session.Resize` when its window gets a different size.
- The daemon does not tell the other attached clients the new size. A client whose window has a different size shows the output as the command made it for the size of the Session.

### Input

An attached client sends its input in `Input` frames, which have no reply and have priority over the output. `Session.Input` is for a client that is not attached, for example a script: the daemon answers after the holder of the Session has the bytes.
The two paths write to the same terminal. The daemon keeps the bytes of one request or of one frame together, and it does not define a sequence between two clients.

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The environment and the window size of a new Session.
- A request that detaches one client and not all the clients of a Session.
- A state for a Session whose command has ended, with the exit status.
- The other services: Project, Worktree, Agent, Team, Task, Events and Ext.
