# Adapter extensions

**Status: DRAFT.** The rules can change in a minor version until version 1.0.0.

An adapter is an extension that takes the hook events of an agent tool and reports the status of the agent to the daemon. This document specifies the payloads between the daemon and an adapter. The schemas are in `proto/nomux/wire/v1/adapter.proto`.
The messages travel in the [extension protocol](https://github.com/nomux-sh/extension-api/blob/main/spec/extension-protocol.md): each one is the payload of a `CallRequest` or of a `CallResponse`. They are not fields of an `Envelope` of the control channel.

## Which extension is the adapter of a tool

- The manifest of the extension names the tools in its `[adapter]` table: `tools = ["testtool"]`. The `extension-api` repository specifies the table.
- The daemon gives the hook events of a tool only to an extension that is installed and enabled, that names the tool, and that has the approved scope `hooks:read`.
- If more than one extension names a tool, the extension with the smallest identifier is the adapter.
- A hook event of a tool without an adapter is dropped.

## Methods of the adapter

The daemon calls these methods of the daemon part of the adapter.

| Method | Argument | Result | Function |
|---|---|---|---|
| `hook` | `HookEvent` | `HookAnswer` | One hook event of a Session. The daemon calls it in the sequence in which the events arrive. |
| `hooks.plan` | empty | `HookInstallPlan` | The change that the hooks of the tool need in the settings file of the tool. |

- The daemon checks the token of the Session before it calls `hook`. `HookEvent` has no token.
- `HookAnswer.output` is the answer of the hook to the tool. In this version the daemon does not wait for the result of `hook` and does not use the output.
- `HookInstallPlan` is data. The client computes the new content of the settings file from it, shows the difference to the user and writes the file only when the user asks for it. An adapter never writes the settings file itself through this method, and the plan never replaces or wraps the command of the tool.

### Install plan

- `settings_paths` has the path of the settings file for each platform. A path starts with `~/` (the home directory of the user) and has no `.` or `..` part. A client refuses a plan without a path for its platform.
- Each entry of `entries` names an array in the JSON settings file with a JSON pointer (RFC 6901) and a value that the array must contain. The client makes each object and the array on the path that the file does not have. A value that the array already contains is not added again.
- A removal removes each value of the entries from its array and nothing else.

## Host services for an adapter

The adapter calls these host services. Each one needs the approved scope `agent:report`.

| Method | Argument | Result | Function |
|---|---|---|---|
| `agent.report` | `AgentReport` | `AgentReportResult` | The status of the Agent run of a Session, with the source `STATUS_SOURCE_ADAPTER`. |
| `agent.child` | `ChildWorkReport` | `AgentReportResult` | The start, a change or the end of one Child work of the Agent run of a Session. |

- The report of an adapter stays until the next report. `AgentReport` with `AGENT_STATE_IDLE` removes the report of the Agent run. A new process of the Session (a restore) removes each report of the adapter.
- `soft` marks a `AGENT_STATE_WORKING` that the adapter only infers, for example from the event of one tool: such an event can arrive after the end of the work. A soft working state does not replace `AGENT_STATE_NEEDS_YOU` of the item, and it does not replace a `AGENT_STATE_DONE` or `AGENT_STATE_IDLE` item that has working Child work below it. A soft working state does not decide the state of the item when a later source reports `AGENT_STATE_NEEDS_YOU` for it.
- A Child work has an identifier and the identifier of its parent: no parent is the Agent run. The tree has 8 levels at most, and the adapter has 64 Child work items at most in one Session. When the limit is reached, a new Child work takes the place of the oldest one that ended. If no Child work ended, the daemon refuses the start.
- An ended Child work keeps its last state, `AGENT_STATE_DONE` or `AGENT_STATE_ERROR`.
- `AgentReport.details` gives the model, the identifier of the session of the tool and the resume command. Clients get them in `AgentRun.details`. A resume command has no prompt and no secret.
- A message, a title, a tool name and each detail is one line of 512 bytes at most without a control character.
- The daemon does not check that the Session got a hook event: an adapter with the scope `agent:report` can report for each Session of the Host. The token proves only where a hook event comes from.

Errors of the host services:

| Condition | Code of the extension protocol |
|---|---|
| The payload is not a valid message. | `ERROR_CODE_INVALID_ARGUMENT` |
| A state, a kind or a change that the daemon does not know, a text that is too long or has a control character, an identifier that is not valid, a parent that is not known, a depth over 8, or 64 Child work items that did not end. | `ERROR_CODE_INVALID_ARGUMENT` |
| No Session with a process has the identifier. | `ERROR_CODE_NOT_FOUND` |
