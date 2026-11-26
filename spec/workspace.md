# Workspace tree

**Status: DRAFT.** The rules can change in a minor version until version 1.0.0.

This document specifies the workspace tree of a Host and the Workspace service of the control protocol: the requests, the errors, the event, the limits and two examples.
The schemas are in `proto/nomux/wire/v1/workspace.proto`. The schemas are the authority for the fields of each message.

## Terms

- **Project.** A repository on a Host. In this version, a Project has a name and a directory. Before this version, the control protocol had no Project.
- **Group.** A node of the workspace tree that holds Projects and Groups. A Group has a name.
- **Workspace tree.** The tree of the Groups and the Projects of one Host. It is not a Project, and it is not a synonym for one.
- **Node.** A Group or a Project in the workspace tree.

## The tree

The daemon owns the workspace tree. It stores the tree on the Host, thus the tree stays the same after a restart of the daemon. Each client gets the same tree from the daemon.

- A Group holds Projects and Groups, to any depth. The depth is not limited. The number of nodes is limited: see [Limits](#limits).
- Each node is in one Group or at the top level. Thus a Project is in one Group at most.
- Each node has a position among its siblings: the nodes with the same parent. The user selects the sequence. The daemon does not sort the nodes.
- Each node has an identifier. The daemon selects the identifier when the node comes into the tree. The identifier does not change when the node gets a new name, a new parent or a new position. The daemon does not use an identifier again for a different node of the Host.
- Two nodes can have the same name, also with the same parent. A client that takes a name from the user, for example the CLI, finds the identifier and refuses a name that has more than one node.
- `Workspace.Tree` gives the tree as a list, not as nested messages: a node comes after its parent, and the siblings are in their sequence. This is the sequence of a depth-first walk. A list has no depth limit of the decoder, thus the tree can be deeper than the nesting limit of a protobuf library.
- If a Group is open or closed in a client is not in the tree. Each client keeps it.

## Projects, Sessions and Worktrees

A Project names a directory on the Host: the root of its repository.

- `Workspace.AddProject` takes an absolute path of a directory. The daemon resolves each symbolic link of the path and stores the result. `WorkspaceProject.directory` is the stored path.
- Two Projects never have the same directory. One Project can have a directory below the directory of a different Project.
- The daemon does not require a git repository. A client that adds a Project from a directory in a repository sends the root of the repository, for example the output of `git rev-parse --show-toplevel`.
- When the directory goes away later, the Project stays in the tree.

A Session relates to a Project by its working directory. No message of the Session service has a field for the Project:

1. Take `Session.working_directory`.
2. A Project matches if its directory is equal to the working directory, or is a directory above it. Compare complete path components: `/src/app` matches `/src/app/web`, but not `/src/application`.
3. If more than one Project matches, the Project with the longest directory is the Project of the Session.
4. If no Project matches, the Session has no Project.

The working directory of a Session does not change, thus the Project of a Session changes only when the tree changes: after `Workspace.AddProject` and `Workspace.RemoveProject`. A Group has no Session of its own: the Sessions of a Group are the Sessions of the Projects in it and in each Group below it.

A Worktree of a Project is a git worktree of the repository of the Project. A Worktree whose directory is below the directory of its Project relates to the Project by the same rule. A Worktree in a different place has no Project in this version: see [Open items](#open-items).

## Status

The workspace tree has no status. A client computes the status of a closed Group from the Agent service:

1. The client takes each Session whose Project is in the Group or in a Group below it.
2. The status of the Group is the most urgent `AgentRun.aggregate` of these Sessions, by the priority of the section "Aggregation" in [control.md](control.md#aggregation).

The tree does not have the status, because:

- The status of a Session changes many times in a minute. The tree changes when the user changes it. If the tree had the status, each change of a status would be a change of the tree: a new revision, a write to the storage of the daemon, and an event with tree data for each client.
- The status already has one source: `Agent.Tree` and the event `AgentRunChanged`. A second copy in the tree can disagree with it.

## Revision

The tree has a revision: an integer that tells a client if its copy of the tree is current.

- A tree that never changed has the revision 0.
- Each request that changes the tree increases the revision by 1 and makes one `WorkspaceChanged` event with the new revision.
- A request that changes nothing succeeds, makes no event and does not change the revision. Examples: a `Workspace.Rename` with the name that the node has, and a `Workspace.Move` to the position that the node has.
- The daemon stores the revision with the tree. The revision never decreases, also after a restart of the daemon. The `seq` of an event starts again after a restart, and the revision does not.

A client keeps its copy of the tree in this way:

1. It sends `Events.Subscribe`, with `EVENT_TYPE_WORKSPACE_CHANGED` in `types` or with no filter.
2. On a different connection, it sends `Workspace.Tree` and keeps the nodes and the revision `R`.
3. For each `WorkspaceChanged` event:
   - If the revision of the event is `R` or smaller, the tree that the client has already includes the change. The client ignores the event.
   - If the revision of the event is `R + 1`, the client applies the change and sets `R` to the revision of the event.
   - If the revision of the event is larger than `R + 1`, the client missed a change. It sends `Workspace.Tree` again.
4. If the client gets an `EventsSnapshot`, it sends `Workspace.Tree` again. A snapshot has no `WorkspaceChanged` event.

## Requests

The Workspace service has the field numbers 800 to 899 in `Request.body`, in `Response.result` and in `Event.body`.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 800 | `Workspace.Tree` | `WorkspaceTreeRequest` | `WorkspaceTreeResponse` | Gives each node of the tree and the revision. |
| 801 | `Workspace.AddProject` | `WorkspaceAddProjectRequest` | `WorkspaceAddProjectResponse` | Adds a Project with a directory and a name, in a Group or at the top level, at a position. Gives the new node. |
| 802 | `Workspace.RemoveProject` | `WorkspaceRemoveProjectRequest` | `WorkspaceRemoveProjectResponse` | Removes a Project from the tree. No file, Session or Worktree changes. |
| 803 | `Workspace.CreateGroup` | `WorkspaceCreateGroupRequest` | `WorkspaceCreateGroupResponse` | Adds an empty Group with a name, in a Group or at the top level, at a position. Gives the new node. |
| 804 | `Workspace.RemoveGroup` | `WorkspaceRemoveGroupRequest` | `WorkspaceRemoveGroupResponse` | Removes a Group. Its content takes the place of the Group in its parent. No node is deleted. |
| 805 | `Workspace.Rename` | `WorkspaceRenameRequest` | `WorkspaceRenameResponse` | Gives a Group or a Project a new name. |
| 806 | `Workspace.Move` | `WorkspaceMoveRequest` | `WorkspaceMoveResponse` | Puts a Group or a Project in a different Group or at the top level, or at a different position among the same siblings. |

Each response to a request that changes the tree has the revision after the change.

### Positions

- `index` in a request is the position among the siblings, from 0. If `index` is absent, or if it is not smaller than the number of siblings, the node is the last.
- For `Workspace.Move`, the siblings are the nodes of the new parent without the node itself. Thus a move to the same parent with the same `index` changes nothing.
- `index` in an event is the position of the node after the change. It is always present.

### Remove a Group

`Workspace.RemoveGroup` removes the Group and keeps its content. The nodes of the Group, in their sequence, take the position of the Group among its siblings. Their parent is now the parent of the Group. The nodes below them do not change. A Group without content goes away.

A client applies the `WorkspaceNodeRemoved` event of a Group with the same rule. The event does not list the nodes that move.

## Errors

| Request | Condition | Code |
|---------|-----------|------|
| Each request with `id` | No node has the identifier. | `ERROR_CODE_NOT_FOUND` |
| Each request with `parent_id` | `parent_id` is not empty, and no node has the identifier. | `ERROR_CODE_NOT_FOUND` |
| Each request with `parent_id` | `parent_id` is the identifier of a Project. Only a Group holds nodes. | `ERROR_CODE_FAILED` |
| `WorkspaceMoveRequest` | `parent_id` is the node itself or a Group below the node. The message says that a Group cannot move into itself or into a Group below it. | `ERROR_CODE_FAILED` |
| `WorkspaceRemoveProjectRequest` | The node is a Group. | `ERROR_CODE_FAILED` |
| `WorkspaceRemoveGroupRequest` | The node is a Project. | `ERROR_CODE_FAILED` |
| `WorkspaceAddProjectRequest`, `WorkspaceCreateGroupRequest`, `WorkspaceRenameRequest` | The name is empty, is longer than 128 bytes, or has a control character. | `ERROR_CODE_FAILED` |
| `WorkspaceAddProjectRequest` | The directory is not an absolute path of a directory, or it is longer than 1024 bytes after the daemon resolved its symbolic links. | `ERROR_CODE_FAILED` |
| `WorkspaceAddProjectRequest` | A Project has the directory already. The message has the identifier of that Project. | `ERROR_CODE_FAILED` |
| `WorkspaceAddProjectRequest`, `WorkspaceCreateGroupRequest` | The tree has 800 nodes. | `ERROR_CODE_FAILED` |
| Each request that changes the tree | The daemon cannot store the tree. The tree does not change. | `ERROR_CODE_FAILED` |

A request that fails does not change the tree and makes no event.

## Event

| Field number in `Event.body` | Event | Type for the filter | Meaning |
|-----------------------------:|-------|---------------------|---------|
| 800 | `WorkspaceChanged` | `EVENT_TYPE_WORKSPACE_CHANGED` | The tree changed. The event has the new revision and one change. |

`WorkspaceChanged.change` is one of:

| Field number | Change | Request | Content |
|-------------:|--------|---------|---------|
| 2 | `WorkspaceNodeAdded` | `Workspace.AddProject`, `Workspace.CreateGroup` | The new node and its position. |
| 3 | `WorkspaceNodeRenamed` | `Workspace.Rename` | The identifier and the new name. |
| 4 | `WorkspaceNodeMoved` | `Workspace.Move` | The identifier, the new parent and the new position. |
| 5 | `WorkspaceNodeRemoved` | `Workspace.RemoveProject`, `Workspace.RemoveGroup` | The identifier. The content of a removed Group takes its place: see [Remove a Group](#remove-a-group). |

A client that gets a change that it does not know sends `Workspace.Tree` again.

## Limits

The response to `Workspace.Tree` goes into one `Envelope`: 1048576 bytes at most. Thus the depth is not limited, but the number of nodes and the size of each node are:

| Item | Limit |
|------|-------|
| Node identifier | 1 to 32 characters from `A-Z`, `a-z`, `0-9`, `-` and `_` |
| Name | 1 to 128 bytes of UTF-8, without a control character |
| Directory of a Project | 1024 bytes after the daemon resolved the symbolic links. This is `PATH_MAX` of macOS, the smaller value of the supported systems. |
| Nodes in the tree | 800 |
| Depth | No limit |

A node with the largest identifier, the largest parent identifier, the largest name and the largest directory has 1232 bytes in the list. 800 such nodes have 985600 bytes. With the `revision` and the fields of the `Response` and of the `Envelope`, the encoded `Envelope` has 985631 bytes. The test `the_largest_tree_goes_into_one_envelope` in `crates/nomux-proto/tests/workspace_wire_format.rs` checks this.

When the tree has 800 nodes, the daemon refuses `Workspace.AddProject` and `Workspace.CreateGroup` with `ERROR_CODE_FAILED`. `Workspace.Rename`, `Workspace.Move` and the two remove requests still work.

## Examples

The identifiers are short in the examples. The tree at the start, at revision 12:

```text
(top level)
├── Client A        group    id g1
│   ├── api         project  id p1  /src/a/api
│   └── web         project  id p2  /src/a/web
├── Client B        group    id g2
│   └── Archive     group    id g3
│       └── old     project  id p3  /src/b/old
└── notes           project  id p4  /home/user/notes
```

`Workspace.Tree` gives the nodes in this sequence: `g1`, `p1`, `p2`, `g2`, `g3`, `p3`, `p4`.

### Move a Group

The user moves the Group `Client A` into the Group `Client B`, as its first node.

1. The client sends `WorkspaceMoveRequest { id: "g1", parent_id: "g2", index: 0 }`.
2. The daemon finds that `g2` is a Group and is not `g1` or a Group below `g1`. The siblings in `g2` without `g1` are `g3`. `index` 0 puts `g1` before `g3`.
3. The daemon stores the tree and answers `WorkspaceMoveResponse { revision: 13 }`.
4. Each subscribed client gets `WorkspaceChanged { revision: 13, moved: { id: "g1", parent_id: "g2", index: 0 } }`.

The tree at revision 13. The nodes of `g1` moved with it:

```text
(top level)
├── Client B        group    id g2
│   ├── Client A    group    id g1
│   │   ├── api     project  id p1
│   │   └── web     project  id p2
│   └── Archive     group    id g3
│       └── old     project  id p3
└── notes           project  id p4
```

A request that moves `g2` into `g1` now fails with `ERROR_CODE_FAILED`, because `g1` is below `g2`.

### Remove a Group with content

At revision 13, the user removes the Group `Client B`.

1. The client sends `WorkspaceRemoveGroupRequest { id: "g2" }`.
2. `g2` is at position 0 at the top level, and its nodes are `g1` and `g3`. They take the position of `g2`, in their sequence: `g1` at position 0, `g3` at position 1. `notes` moves to position 2.
3. The daemon stores the tree and answers `WorkspaceRemoveGroupResponse { revision: 14 }`.
4. Each subscribed client gets `WorkspaceChanged { revision: 14, removed: { id: "g2" } }` and applies the same rule.

The tree at revision 14. No Project and no Group except `g2` went away:

```text
(top level)
├── Client A        group    id g1
│   ├── api         project  id p1
│   └── web         project  id p2
├── Archive         group    id g3
│   └── old         project  id p3
└── notes           project  id p4
```

## Open items

These items are not decided. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

- The default agent and the color of a Project.
- A Worktree whose directory is not below the directory of its Project. The Worktree service relates it to its Project.
- A field of `Session` with the Project, if a client cannot compute it from the working directory.
- A request that adds or moves more than one node in one change.
