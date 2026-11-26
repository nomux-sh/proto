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
- The daemon sends an `Event` only on a connection that asked for events with `Events.Subscribe`. "Events" has the rules.
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
| Agent | 500 to 599 | `agent.proto` |
| Events | 600 to 699 | `events.proto` |
| Hooks | 700 to 799 | `hooks.proto` |

An event of a service has a field number of the block of that service in `Event.body`.

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
| 310 | `Session.Memory` | `SessionMemoryRequest` | `SessionMemoryResponse` | Gives the memory that the processes of each Session use. |

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

### Memory

`Session.Memory` gives the memory that the processes of each Session use at the time of the request.

- The daemon measures the memory when it gets the request. It keeps no value between two requests, and it measures nothing when no client asks. A client that shows the values again sends the request again.
- The response has one entry for each Session that has a holder: the states `SESSION_STATE_RUNNING` and `SESSION_STATE_UNREACHABLE`. A Session without a process has no entry. A Session whose processes end during the request can have no entry.
- `command_bytes` is the memory of the process tree of the command: the command and each process that descends from it. `command_processes` is the number of these processes. A process that left the tree is not in the value: the operating system gave it a different parent after its own parent ended.
- `holder_bytes` is the memory of the holder. It is not a part of `command_bytes`.
- `measure` says how the daemon measured the two values of one entry:

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `MEMORY_MEASURE_UNSPECIFIED` | The daemon did not set the field. The client treats the measure as unknown. |
| 1 | `MEMORY_MEASURE_PROPORTIONAL` | The proportional set size (PSS). A page that more than one process uses is divided between these processes, thus a sum of values counts no page two times. |
| 2 | `MEMORY_MEASURE_RESIDENT` | The resident set size (RSS). A page that more than one process uses is in the value of each of them, thus a sum of values can be larger than the memory in use. |

- The daemon uses `MEMORY_MEASURE_PROPORTIONAL` for an entry only if the operating system gives that value for each process of the entry. The entries of one response can have different measures.
- A client treats a measure that it does not know as unknown.

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

### Agent

The Agent service gives the status of the Agent run of each Session and of its Child work. In this version, each Session has one Agent run: what runs in the Session.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 500 | `Agent.Tree` | `AgentTreeRequest` | `AgentTreeResponse` | Gives the status of each Session that is not ended, with its Child work if the request asks for it, and the most urgent status of the Host. |
| 501 | `Agent.Explain` | `AgentExplainRequest` | `AgentExplainResponse` | Gives the status of one Session and what each source says about it. |
| 502 | `Agent.SetStatus` | `AgentSetStatusRequest` | `AgentSetStatusResponse` | A program in a Session reports the status of that Session: the self-report. |

An Agent run or a Child work has one of these states:

| Value | Name | Meaning |
|------:|------|---------|
| 0 | `AGENT_STATE_UNSPECIFIED` | The daemon did not set the field. The client treats the state as unknown. |
| 1 | `AGENT_STATE_IDLE` | Nothing runs, and nothing waits for the user. |
| 2 | `AGENT_STATE_WORKING` | Work runs. |
| 3 | `AGENT_STATE_DONE` | Work is complete. `seen` says if the user saw the result. |
| 4 | `AGENT_STATE_NEEDS_YOU` | The work cannot continue until the user does something. `kind` says what: `NEEDS_YOU_KIND_PERMISSION`, `NEEDS_YOU_KIND_QUESTION` or `NEEDS_YOU_KIND_AUTH`, or `NEEDS_YOU_KIND_UNSPECIFIED` if no source says it. |
| 5 | `AGENT_STATE_ERROR` | The work failed and stopped. `seen` says if the user saw the failure. |

A client treats a state that it does not know as unknown.

#### Sources

Each part of an `AgentStatus` has its source: the state with its kind, the message, the title, the progress and the name of the tool. Thus a client can tell a report of a program from a guess of the daemon, and it must not show a guess as a fact.

| Value | Name | The status comes from |
|------:|------|-----------------------|
| 0 | `STATUS_SOURCE_UNSPECIFIED` | The daemon did not set the field. |
| 1 | `STATUS_SOURCE_NONE` | No source. The state is `AGENT_STATE_IDLE`. |
| 10 | `STATUS_SOURCE_ADAPTER` | An adapter extension, from the events of the agent tool: [adapter.md](adapter.md). |
| 20 | `STATUS_SOURCE_PROGRAM_STATUS` | A report of a program in the terminal stream of the Session: the Program Status Protocol (OSC 7501). |
| 30 | `STATUS_SOURCE_SELF_REPORT` | `Agent.SetStatus`. |
| 40 | `STATUS_SOURCE_SESSION_RESTORE` | The Session lost its process while work ran or waited for the user. The daemon shows that work as `AGENT_STATE_DONE`. |
| 50 | `STATUS_SOURCE_PROMPT_MARKS` | A guess from the prompt marks of the shell (OSC 133). |
| 60 | `STATUS_SOURCE_FOREGROUND_PROGRAM` | A guess from the foreground process group of the terminal of the Session. |

- The sources decide in the sequence of their values from 10 on: the first source that reports a state gives the state and the kind. The values have room between them, because a later version adds sources.
- One exception: a soft `AGENT_STATE_WORKING` of the adapter does not decide when a later source reports `AGENT_STATE_NEEDS_YOU` for the same item. The later source decides then. The title and the tool name of the adapter still count. [adapter.md](adapter.md) says when a state is soft.
- A text or the progress comes from the first source that has it and that reports the same state as the source of the state. A text of a source that reports a different state is not shown: it describes a different state.
- A message has 512 bytes at most. The daemon cuts a longer message of a program at the end of a character.
- A client treats a source that it does not know as a source without a name. It does not treat it as a guess or as a report.
- `AgentExplainResponse.sources` has one `SourceReport` for each source that the daemon has, in the sequence in which they decide, also for a source that reports nothing. The `note` of a report says why the source reports nothing, or what its report is based on.

#### Child work

A `ChildWork` is a subagent, a teammate or a background task below an Agent run or below a Child work. The tree has 8 levels of Child work at most.

- `status` is the status of the Child work. It is absent if no source reports the Child work and it is in the tree only because a Child work below it has a status.
- `children` are in the sequence of their names.

#### Details

`AgentRun.details` has what an adapter says about the Agent run beyond its status: the model, the identifier of the session of the agent tool and a command that continues that session. It is absent if no adapter gave details. The source is always `STATUS_SOURCE_ADAPTER`.

#### Aggregation

`aggregate` is the most urgent status of an item and of each Child work below it. `AgentTreeResponse.aggregate` is the most urgent status of the Host, and `attention_session_id` names the Session that has it.

The priority, from the most urgent:

1. `AGENT_STATE_NEEDS_YOU`.
2. `AGENT_STATE_ERROR` with `seen` false.
3. `AGENT_STATE_DONE` with `seen` false.
4. `AGENT_STATE_WORKING`.
5. Each other status: `AGENT_STATE_IDLE`, and `AGENT_STATE_DONE` or `AGENT_STATE_ERROR` with `seen` true.

With the same priority, the status with the smallest `since_ms` is more urgent: the one that waits longest. An aggregate is a copy of one status of the tree, with the source of that status.

#### Since and seen

- `since_ms` is the time at which the daemon saw the current state and kind of an item for the first time. A change of the message or of the progress does not change it.
- `seen` starts as false with each new state. A `Session.Attach` to the Session sets it for the Agent run and for each Child work: the user looks at the Session.
- The daemon keeps `since_ms` and `seen` in memory. After a restart of the daemon, `since_ms` is the time at which the new daemon read the status, and `seen` is false.

#### Sessions without a process

A Session in the state `SESSION_STATE_DORMANT` has no process, thus it has no work that runs and no work that waits for the user. When a Session becomes dormant, the daemon changes each `AGENT_STATE_WORKING` and `AGENT_STATE_NEEDS_YOU` status of it to `AGENT_STATE_DONE` with the source `STATUS_SOURCE_SESSION_RESTORE`. A Session that was dormant when the daemon started has the state `AGENT_STATE_IDLE` with the source `STATUS_SOURCE_NONE`. After a restore, the status comes from the sources of the new process.

A Session in the state `SESSION_STATE_ENDED` has no Agent run.

#### Self-report

A program in a Session reports the status of its Session with `Agent.SetStatus`. Each Session gets three environment variables for this: `NOMUX_SESSION_ID` (the session identifier), `NOMUX_SOCK` (the path of the socket of the daemon) and `NOMUX_TOKEN` (the token of the Session).

- The token is a random value that the holder of the Session makes at its start. Only the processes of the Session and the daemon know it. It proves that a request comes from a process of that Session, thus a program cannot report for a different Session.
- A daemon never puts a token in a response, in an event or in a log. A client never prints one.
- The self-report stays until the next `Agent.SetStatus` of the Session. A request with `AGENT_STATE_IDLE` removes it.
- A self-report decides only if no source before it reports a state.

Errors of the Agent service:

| Request | Condition | Code |
|---------|-----------|------|
| `AgentExplainRequest`, `AgentSetStatusRequest` | No Session has the identifier, or the Session has the state `SESSION_STATE_ENDED`. | `ERROR_CODE_NOT_FOUND` |
| `AgentSetStatusRequest` | The token is empty or is not the token of the Session, or the daemon has no token of the Session: its holder is too old to make one, or the Session has no process. | `ERROR_CODE_PERMISSION_DENIED` |
| `AgentSetStatusRequest` | The state is `AGENT_STATE_UNSPECIFIED` or a value that the daemon does not know, or the message is longer than 512 bytes, has more than one line or has a control character. | `ERROR_CODE_FAILED` |

### Events

The Events service gives a client the changes on a Host as a stream of `Event` messages. The daemon does not poll for a change, and a client does not poll the daemon.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 600 | `Events.Subscribe` | `EventsSubscribeRequest` | `EventsSubscribeResponse` | Starts the stream of the events of the Host on this connection. |

- `EventsSubscribeResponse` is the last response on its connection. After it, the daemon sends only `Event` messages there, and it reads no request. To end the subscription, the client closes the connection.
- Each event has a sequence number `seq`. The first event of a daemon has the number 1, and each event has the number after the event before it. The numbers start again when the daemon starts again.
- The daemon keeps the last events in memory, with a limit of their number and of their bytes: 1024 events, or fewer when the events are large. An event has the Agent run of a Session, and a program in the Session decides its size. A client that connects again sends the `seq` after the last one that it has as `from_seq`, and it gets each event from there on, one time, if the daemon still has that event.
- With `from_seq` 0, the client gets only the events after the request.
- If the daemon does not have the event `from_seq` any more, or if `from_seq` is larger than the number after the newest event (the daemon started again), the client gets a snapshot: one `EventsSnapshot` event, then one `AgentRunChanged` event for each Session that is not ended, and then the events that follow. The events of a snapshot have the `seq` of the newest event at that time. A client that gets an `EventsSnapshot` discards the state that it has.
- A client that is slow gets the events later. If it is too far behind, it gets a snapshot as above. The daemon keeps no queue for one client.
- With `types`, the client gets only the events of these types. The filter does not apply to `EventsSnapshot`.

| Field number in `Event.body` | Event | Type for the filter | Meaning |
|-----------------------------:|-------|---------------------|---------|
| 500 | `AgentRunChanged` | `EVENT_TYPE_AGENT_RUN_CHANGED` | A part of the status of a Session or of its Child work changed. The event has the Agent run after the change. |
| 400 | `ExtensionEvent` | `EVENT_TYPE_EXTENSION_EVENT` | The daemon part of an extension sent an event. The daemon drops an event whose payload is larger than 65536 bytes. |
| 501 | `AgentRunRemoved` | `EVENT_TYPE_AGENT_RUN_REMOVED` | The Session ended or was removed. |
| 600 | `EventsSnapshot` | none | The state of the Host follows. |

A client ignores an event whose body it does not know.

The daemon does not make an event for a change that only the source `STATUS_SOURCE_FOREGROUND_PROGRAM` sees: the operating system does not tell the daemon when the foreground process group of a terminal changes. The daemon reads that source for `Agent.Tree` and `Agent.Explain`, and when a different source changes.

### Hooks

The Hooks service takes the hook events of agent tools and gives the clients what they need to install the hooks of a tool. [adapter.md](adapter.md) specifies the adapter extensions that take the events.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 700 | `Hooks.Event` | `HookEventRequest` | `HookEventResponse` | A hook of an agent tool in a Session sends one event of the tool. |
| 701 | `Hooks.Plan` | `HooksPlanRequest` | `HooksPlanResponse` | Gives the change that the hooks of a tool need in its settings file, as the adapter of the tool describes it. |
| 702 | `Hooks.Adapters` | `HooksAdaptersRequest` | `HooksAdaptersResponse` | Gives each tool that has an adapter, and its adapter. |

- `nomux hook <tool> <event>` sends `Hooks.Event`. It takes the Session, the socket and the token from `NOMUX_SESSION_ID`, `NOMUX_SOCK` and `NOMUX_TOKEN`, as `Agent.SetStatus` does. The token rules of the self-report apply.
- The daemon answers `Hooks.Event` at once and gives the event to the adapter of the tool later, in the sequence of arrival. It keeps 256 events or 4 MiB of payload at most that the adapter did not take; it drops an event over the limit. An event of a tool without an adapter is dropped. The response is the same in these cases: a hook must not depend on the adapter.
- `HookEventResponse.output` is what the hook writes to its standard output. It is empty in this version. A later version fills it for an event that waits for a decision.
- A hook must never block the tool. A client that does not get the response in its own time limit ends without an output.
- `Hooks.Plan` calls the method `hooks.plan` of the adapter. The daemon does not read or write a settings file: the client computes the change, shows it and writes it on the computer of the tool.

Errors of the Hooks service:

| Request | Condition | Code |
|---------|-----------|------|
| `HookEventRequest` | The tool or the event is not a valid name, or the payload is larger than 262144 bytes. | `ERROR_CODE_FAILED` |
| `HookEventRequest` | No Session that is not ended has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `HookEventRequest` | The token is empty or is not the token of the Session, or the daemon has no token of the Session. | `ERROR_CODE_PERMISSION_DENIED` |
| `HooksPlanRequest` | No tool with this name has an adapter. | `ERROR_CODE_NOT_FOUND` |
| `HooksPlanRequest` | The adapter did not answer, answered with an error, or answered with a payload that is not a `HookInstallPlan`. | `ERROR_CODE_FAILED`, or `ERROR_CODE_TIMEOUT` if it did not answer in time |

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The environment and the window size of a new Session.
- A request that detaches one client and not all the clients of a Session.
- A request that removes an ended Session before its retention time is over, and a request that starts the command of an ended Session again.
- The name of the environment profile of a Session.
- The other services: Project, Worktree, Team and Task.
- Agent: more than one Agent run in a Session, the kind of a Child work (subagent, teammate, background task), and a self-report for a Child work.
- Events: a request that marks a Session as seen without an attach, the events of the other services, and an event log that stays after a restart of the daemon.
- Ext: the installation of an extension from a registry, the update of an extension, and events from the daemon to an extension.
- Hooks: an answer of a hook with a decision of the user, and settings files that are not JSON.
