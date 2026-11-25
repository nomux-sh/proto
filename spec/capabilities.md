# Capability flag registry

**Status: DRAFT.** The rules can change until ADR-0002 is accepted.

A capability flag is the name of one optional feature of the nomux protocol.
This document is the registry of all flag names.

## How flags are used

1. The client sends its flags in `Hello.capabilities`.
2. The daemon sends its flags in `Welcome.capabilities`.
3. A feature is active on a connection only if the two lists contain its flag.
4. A peer ignores a flag name that it does not know.

A flag does not replace the protocol version. The version rule is in [compatibility.md](compatibility.md). The version tells which messages a peer can decode. A flag tells which optional behavior a peer implements.

## Rules for flag names

- A name contains only the characters `a` to `z`, `0` to `9`, and `_`.
- A name starts with a letter.
- A name has a maximum length of 64 bytes.
- A name is registered in this document before a release uses it.
- A name is never used again for a different meaning. If the meaning must change, register a new name.
- A name is never removed from this registry. A flag that is no longer used gets the state `retired`.

## States

| State      | Meaning |
|------------|---------|
| `reserved` | The name is taken. The meaning is not specified yet. A peer must not send the flag. |
| `active`   | The meaning is specified. A peer can send the flag. |
| `retired`  | A peer must not send the flag. The name cannot be used again. |

## Registry

"Since" is the first control protocol version in which a peer can send the flag.

| Name                | State      | Since | Meaning |
|---------------------|------------|-------|---------|
| `waits`             | `reserved` | —     | Not specified yet. The name is reserved for wait operations that are driven by events. |
| `pane_state_events` | `reserved` | —     | Not specified yet. The name is reserved for events about pane state. |

No flag is `active` in this version of the registry.

## How to add a flag

1. Add a row to the registry in the same pull request that specifies the feature.
2. Set the state to `active` and set "Since" to the control protocol version that first includes the feature.
3. If the feature adds a terminal frame kind or a frame flag bit, also update [frame.md](frame.md).
