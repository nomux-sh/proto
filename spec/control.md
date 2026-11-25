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
| Ext | 400 to 499 | `ext.proto` |

### Handshake

| Field number | Request | Response |
|-------------:|---------|----------|
| 100 | `Hello` | `Welcome` |

### Host

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 200 | `Host.Info` | `HostInfoRequest` | `HostInfoResponse` | Gives the product version of the daemon, its process ID and the number of Sessions. Ended Sessions are not in the number. |
| 201 | `Host.Stop` | `HostStopRequest` | `HostStopResponse` | Stops the daemon. The daemon sends the response, closes each connection and exits. No Session ends. |

### Session

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 300 | `Session.Create` | `SessionCreateRequest` | `SessionCreateResponse` | Starts a new Session and gives its identifier, its state and the process ID of its command. |
| 301 | `Session.List` | `SessionListRequest` | `SessionListResponse` | Gives the Sessions on the Host. Ended Sessions are in the response only if the request asks for them. |
| 302 | `Session.Kill` | `SessionKillRequest` | `SessionKillResponse` | Ends a Session. The daemon answers after the command of the Session has ended. For a dormant or a damaged Session, the daemon removes the Session and its state. |
| 303 | `Session.Attach` | `SessionAttachRequest` | `SessionAttachResponse` | Connects the client to the terminal of a Session. After the response, the client gets the screen and the output in terminal frames. |
| 304 | `Session.Detach` | `SessionDetachRequest` | `SessionDetachResponse` | Ends each attach to a Session and gives the number of clients that were attached. The Session continues. |
| 305 | `Session.Input` | `SessionInputRequest` | `SessionInputResponse` | Writes bytes to the terminal of a Session. |
| 306 | `Session.Resize` | `SessionResizeRequest` | `SessionResizeResponse` | Changes the window size of a Session. |
| 307 | `Session.Start` | `SessionStartRequest` | `SessionStartResponse` | Starts the command of a dormant Session again. |
| 308 | `Session.Capture` | `SessionCaptureRequest` | `SessionCaptureResponse` | Gives output bytes of a Session that are before a position: the scrollback. |
| 309 | `Session.SetRestorePolicy` | `SessionSetRestorePolicyRequest` | `SessionSetRestorePolicyResponse` | Sets when the daemon starts the command of a Session again. |

Errors of the Session service:

| Request | Condition | Code |
|---------|-----------|------|
| `SessionCreateRequest` | The working directory is not an absolute path of a directory, or the command cannot start. | `ERROR_CODE_FAILED` |
| `SessionCreateRequest` | The Session was not ready in the permitted time. | `ERROR_CODE_TIMEOUT` |
| Each request with an identifier | No Session has the identifier. | `ERROR_CODE_NOT_FOUND` |
| Each request with an identifier, but not `SessionCaptureRequest` | The Session has the state `SESSION_STATE_ENDED`. | `ERROR_CODE_NOT_FOUND` |
| `SessionKillRequest` | The command did not end in the permitted time. | `ERROR_CODE_TIMEOUT` |
| `SessionAttachRequest`, `SessionInputRequest`, `SessionResizeRequest` | The daemon has no connection to the holder of the Session (state `SESSION_STATE_UNREACHABLE`). | `ERROR_CODE_FAILED` |
| `SessionAttachRequest`, `SessionResizeRequest` | The window size is not valid. | `ERROR_CODE_FAILED` |
| `SessionInputRequest` | The command of the Session has ended. | `ERROR_CODE_FAILED` |
| `SessionInputRequest`, `SessionResizeRequest` | The Session is dormant. | `ERROR_CODE_FAILED` |
| `SessionAttachRequest` | The Session is dormant and its restore policy is `RESTORE_POLICY_MANUAL`. | `ERROR_CODE_FAILED` |
| `SessionAttachRequest`, `SessionStartRequest` | The Session is dormant and its command cannot start. | `ERROR_CODE_FAILED` |
| `SessionAttachRequest`, `SessionStartRequest`, `SessionInputRequest`, `SessionResizeRequest`, `SessionSetRestorePolicyRequest` | The Session has the state `SESSION_STATE_DAMAGED`. | `ERROR_CODE_FAILED` |
| `SessionSetRestorePolicyRequest` | The policy is `RESTORE_POLICY_UNSPECIFIED` or a value that the daemon does not know, or the daemon cannot store the policy. | `ERROR_CODE_FAILED` |

A Session has one of these states:

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `SESSION_STATE_UNSPECIFIED` | The daemon did not set the field. The client treats the state as unknown. |
| 1 | `SESSION_STATE_RUNNING` | The command of the Session runs. |
| 2 | `SESSION_STATE_UNREACHABLE` | The holder of the Session runs, but the daemon has no connection to it. The daemon tries to connect again. |
| 3 | `SESSION_STATE_DORMANT` | The Session has no process: its holder is gone, and its command did not end. This is the state of each Session after a restart of the Host. The daemon has the description of the Session and its output. |
| 4 | `SESSION_STATE_ENDED` | The command of the Session ended. The daemon keeps the Session and its output for a retention time, and then it removes them. |
| 5 | `SESSION_STATE_DAMAGED` | The daemon cannot read the description of the Session. `Session.damage` has the reason. The Session cannot start again. |

A client treats a state that it does not know as unknown.

### Fields of a Session

`Session` has the description of a Session: the command, the working directory, the window size, the start time, the epoch and the restore policy. The daemon gives each field that it knows, also for a Session without a process.

- The description never has the environment of the command or a value of it.
- `pid` is not 0 only in the state `SESSION_STATE_RUNNING`.
- `exit` is set only in the state `SESSION_STATE_ENDED`. It has the time of the end, and the exit code or the signal if the daemon knows it.
- `damage` is not empty when the daemon cannot read a part of the state files of the Session. In the state `SESSION_STATE_DAMAGED`, it is the reason for the state. In a different state, a part of the output of the Session is gone, and the Session can be used.

### Restore

The daemon keeps the description and the output of each Session on the Host. Thus a Session whose holder is gone, for example after a restart of the Host, is not lost: it is dormant. A restore starts its command again.

- A restore starts a new holder with the command, the working directory and the window size that the Session had. The Session keeps its identifier and gets the next epoch. The processes of the earlier epoch do not come back: only the output stays.
- The restore policy of a Session says when the daemon does the restore:

| Value | Name | The daemon starts the command again |
|------:|------|-------------------------------------|
| 1 | `RESTORE_POLICY_AUTO` | When the daemon starts, and for a `SessionAttachRequest` or a `SessionStartRequest`. |
| 2 | `RESTORE_POLICY_ON_FOCUS` | For a `SessionAttachRequest` or a `SessionStartRequest`. This is the policy of a Session that has none. |
| 3 | `RESTORE_POLICY_MANUAL` | Only for a `SessionStartRequest`. |

- `Session.SetRestorePolicy` sets the policy of a Session whose command runs or of a dormant Session. The daemon stores it with the state of the Session.
- `Session.Start` on a Session whose command runs succeeds and changes nothing.
- [frame.md](frame.md) specifies what the client of the `SessionAttachRequest` that starts the command gets: the end of the earlier output, a `Restored` frame and the new screen.

### Capture

`Session.Capture` gives output bytes of a Session without an attach: the scrollback. It works for a Session in each state, also for a dormant and an ended Session.

- The response has the last output bytes before `before_seq` that the daemon has without a gap: `max_bytes` at most, and 262144 bytes at most. `seq` is the sequence number of the first byte.
- Without `before_seq`, the response ends at the end of the output of the epoch.
- To read the bytes before a response, a client sends the request again with `before_seq` equal to the `seq` of that response.
- If `data` is empty, the daemon has no output of that epoch before `before_seq`. The output of the epoch before it is in the epoch with the next smaller number: a client asks for that epoch without `before_seq`.
- With the epoch `0`, the daemon uses the newest epoch of the Session, and the response has its number.
- The bytes are the output as the command wrote it. A response can start or end in the middle of an escape sequence or of a UTF-8 character.

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
- A dormant Session keeps its window size. A restore starts the command with that size, or with the size of the `SessionAttachRequest` that starts it.
- A client that is attached sends `Session.Resize` when its window gets a different size.
- The daemon does not tell the other attached clients the new size. A client whose window has a different size shows the output as the command made it for the size of the Session.

### Input

An attached client sends its input in `Input` frames, which have no reply and have priority over the output. `Session.Input` is for a client that is not attached, for example a script: the daemon answers after the holder of the Session has the bytes.
The two paths write to the same terminal. The daemon keeps the bytes of one request or of one frame together, and it does not define a sequence between two clients.

### Ext

The Ext service manages the extensions of a Host and calls the methods of their daemon parts. The [extension protocol](https://github.com/nomux-sh/extension-api/blob/main/spec/extension-protocol.md) specifies how the daemon talks to a daemon part.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 400 | `Ext.List` | `ExtListRequest` | `ExtListResponse` | Gives the installed extensions: identifier, name, version, state, the reason for the state and the approved permissions. |
| 401 | `Ext.Inspect` | `ExtInspectRequest` | `ExtInspectResponse` | Validates the package in a directory on the Host and gives the permissions that its manifest asks for. Changes nothing. |
| 402 | `Ext.Install` | `ExtInstallRequest` | `ExtInstallResponse` | Installs the package in a directory on the Host with the permissions that the user approved, and enables the extension. Starts no process. |
| 403 | `Ext.Remove` | `ExtRemoveRequest` | `ExtRemoveResponse` | Stops an extension and removes its package, its state and its data. |
| 404 | `Ext.SetEnabled` | `ExtSetEnabledRequest` | `ExtSetEnabledResponse` | Enables or disables an extension and gives the extension after the change. The daemon stops the process of an extension that it disables. |
| 405 | `Ext.Call` | `ExtCallRequest` | `ExtCallResponse` | Calls a method of the daemon part of an extension and gives the answer of the extension. |

An extension has one of these states:

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `EXTENSION_STATE_UNSPECIFIED` | The daemon did not set the field. The client treats the state as unknown. |
| 1 | `EXTENSION_STATE_STOPPED` | The extension is enabled and its daemon part does not run. The next `Ext.Call` starts it. |
| 2 | `EXTENSION_STATE_STARTING` | The process of the daemon part started and is not ready. |
| 3 | `EXTENSION_STATE_RUNNING` | The daemon part runs. |
| 4 | `EXTENSION_STATE_FAILED` | The daemon part failed and does not run. `state_reason` has the failure. After a pause, the next `Ext.Call` starts it again. |
| 5 | `EXTENSION_STATE_DISABLED` | The daemon does not start the extension. `state_reason` says why: the user disabled it, or the daemon disabled it after failures. |

#### Installation

An installation has two steps, thus no extension runs before the user saw and approved its permissions.

1. The client sends `Ext.Inspect` with the directory of the package. The daemon validates the manifest and answers with the identifier, the name, the version and the permissions that the manifest asks for. The daemon installs nothing.
2. The client shows the permissions to the user. The user approves all of them, a part of them, or none, or stops the installation.
3. The client sends `Ext.Install` with the same directory and the approved permissions in `granted`. The daemon validates the package again, copies it and enables the extension.

- The extension gets the permissions in `granted` and no others, also when its manifest asks for more. A request without `granted` approves no permission.
- The daemon refuses an `Ext.Install` in which `granted` has a permission that the manifest does not ask for. An approved directory is equal to an item of `fs` in the manifest, or the item starts with `${worktree}` and the directory ends with the rest of the item.
- `Ext.Install` starts no process. Only `Ext.Call` starts the daemon part of an extension, and only for an extension that is installed and enabled.
- A client must not send `Ext.Install` before the user approved the permissions of step 1. A client without a user interface, for example a script, gets the approval from its arguments.

#### Call

`Ext.Call` has two kinds of failure:

- **The extension answered with an error.** The response is an `ExtCallResponse` with an `ExtCallError`. The code is the error code of the extension protocol, and the values of `ExtCallErrorCode` are equal to the values of that protocol.
- **The daemon got no answer from the extension.** The response is an `Error` of the control protocol, with a code from the table below.

A response with an empty `payload` is a result: the method gave zero bytes.

The payload of a call and the payload of a result go in one `Envelope`, thus each of them has less than 1 MiB. A frame of the extension protocol can be larger: the daemon answers a result that does not go into an `Envelope` with `ERROR_CODE_FAILED`.

Errors of the Ext service:

| Request | Condition | Code |
|---------|-----------|------|
| `ExtInspectRequest`, `ExtInstallRequest` | The directory is not an absolute path, the manifest is not valid, the package does not have the program of its daemon part, or the daemon does not supply the extension API version that the manifest asks for. For a manifest that is not valid, the message has the path of each field that is not valid. | `ERROR_CODE_FAILED` |
| `ExtInstallRequest` | `granted` has a permission that the manifest does not ask for, or an extension with the identifier is already installed. | `ERROR_CODE_FAILED` |
| `ExtRemoveRequest`, `ExtSetEnabledRequest`, `ExtCallRequest` | No extension has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `ExtCallRequest` | The extension is disabled, has no daemon part, cannot start, or its process failed before it answered. | `ERROR_CODE_FAILED` |
| `ExtCallRequest` | The extension did not answer in the permitted time. | `ERROR_CODE_TIMEOUT` |
| `ExtCallRequest` | The payload or the result does not go into one `Envelope`. | `ERROR_CODE_FAILED` |

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The environment and the window size of a new Session.
- A request that detaches one client and not all the clients of a Session.
- A request that removes an ended Session before its retention time is over, and a request that starts the command of an ended Session again.
- The name of the environment profile of a Session.
- The other services: Project, Worktree, Agent, Team, Task and Events.
- Ext: the installation of an extension from a registry, the update of an extension, and the events of the extensions.
