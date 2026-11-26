# Worktrees

**Status: DRAFT.** The rules can change in a minor version until version 1.0.0.

This document specifies the Worktrees of the Projects of a Host and the Worktree service of the control protocol: the states, the flags, the reasons why a removal is refused, the sizes, the request, the errors, and the method of the git extension that gives the git facts.
The schemas are in `proto/nomux/wire/v1/worktree.proto` and `proto/nomux/wire/v1/worktree_facts.proto`. The schemas are the authority for the fields of each message.

This version only reads. The daemon changes no file of a repository and no file of its git directory, and it runs no `git` command. A later version adds the removal: the reasons of [Refusals](#refusals) are the checks of that removal.

## Terms

- **Worktree.** A directory that `git worktree list` gives for the repository of a Project: the main worktree or a linked worktree. A worktree that an agent tool made is an ordinary linked worktree.
- **Base branch.** The branch that the merge rules compare with. See [Merge rules](#merge-rules).
- **Build output.** A directory that a build or an install makes again, for example `target` or `node_modules`. The daemon decides with its build-output rules. The rules are data of the daemon and not a part of the protocol.

## The Worktrees of a Project

- The Worktrees of a Project are the worktrees of the repository of the directory of the Project, wherever the directory of a Worktree is. Two Projects in the same repository have the same Worktrees.
- A Session belongs to the Worktree with the longest directory that contains its working directory. The comparison uses the rule of [workspace.md](workspace.md#projects-sessions-and-worktrees): the resolved working directory, complete path components. A Worktree can be below the main worktree; then a Session in it belongs to it and not to the main worktree.
- The daemon gets the git facts from the git extension `nomux.git`: [The method `worktree_facts`](#the-method-worktree_facts). The daemon has no git code. Without the extension, or when the approved directories of the extension do not contain the directory of the Project, the Project has no Worktree and `ProjectWorktrees.unavailable_reason` says why and what the user can do.
- The daemon asks the extension for each `Worktree.List`. It keeps no list of Worktrees. It keeps only the sizes: see [Sizes](#sizes).
- nomux never fetches. `ProjectWorktrees.last_fetch_at_ms` tells the user how old the remote-tracking branches are.

## States

A Worktree has one state. The daemon applies the rows in this sequence and gives the first that matches.

| State | Condition |
|---|---|
| `WORKTREE_STATE_ACTIVE` | A Session that is not ended has its working directory in the Worktree. A dormant Session counts. |
| `WORKTREE_STATE_FINISHED` | A merge rule matches the branch (`merged_by` is not `WORKTREE_MERGED_BY_UNSPECIFIED`). |
| `WORKTREE_STATE_STALE` | `last_activity_at_ms` is older than the stale time. The stale time is 14 days in this version. |
| `WORKTREE_STATE_IDLE` | Each other Worktree. |

The main worktree and a bare repository get a state by the same rules.

## Flags

The flags are facts of the Worktree, independent of the state. Each flag has one source:

| Flag | Source |
|---|---|
| `WORKTREE_FLAG_DIRTY` | `WorktreeFacts.is_dirty` |
| `WORKTREE_FLAG_UNPUSHED` | `WorktreeFacts.unpushed_commits` is larger than 0 |
| `WORKTREE_FLAG_DETACHED_COMMITS` | `WorktreeFacts.has_detached_commits` |
| `WORKTREE_FLAG_LOCKED` | `WorktreeFacts.is_locked` |
| `WORKTREE_FLAG_SUBMODULE` | `WorktreeFacts.has_submodules` |
| `WORKTREE_FLAG_IGNORED_USER_FILES` | An ignored path of `WorktreeFacts.ignored_paths` that the build-output rules of the daemon do not accept |
| `WORKTREE_FLAG_BUSY` | The daemon: a process of the Host has its current directory in the Worktree. The daemon reads the current directory of each process where the operating system lets it |
| `WORKTREE_FLAG_MISSING` | `WorktreeFacts.is_missing` |
| `WORKTREE_FLAG_OTHER_FILE_SYSTEM` | The daemon: the directory has a different device than its parent directory |
| `WORKTREE_FLAG_UPSTREAM_GONE` | `WorktreeFacts.upstream_gone` |

## Merge rules

The extension finds the base branch in this sequence: the branch that `refs/remotes/origin/HEAD` names, else `main`, else `master`. For `main` and `master`, the remote-tracking branch of `origin` comes before the local branch. A repository without these branches has no base branch, and only the rule `WORKTREE_MERGED_BY_UPSTREAM_GONE` can match.

The extension applies the rules in this sequence and gives the first that matches. A detached `HEAD`, a bare repository and the branch that is the base branch match no rule.

| Rule | Condition | Finds |
|---|---|---|
| `WORKTREE_MERGED_BY_ANCESTOR` | The base branch has the commit of `HEAD`. | A merge commit, a fast-forward |
| `WORKTREE_MERGED_BY_TREE` | A merge of `HEAD` into the base branch, computed in memory, has no conflict and gives the tree of the base branch. | A rebase merge, a squash merge without a later change of the same lines |
| `WORKTREE_MERGED_BY_UPSTREAM_GONE` | The branch has an upstream branch in its configuration, and its remote-tracking branch does not exist. | Each merge of a pull request, after a fetch with prune, if the forge deletes the branch. Also a branch that a person deleted without a merge |

The merge in memory writes no object. The rules use no network.

## Refusals

`WorktreeInfo.refusals` says why a removal of the directory of the Worktree is refused. Such a removal keeps the branch. The daemon gives the reasons in this sequence, thus the first reason is the most important:

| Reason | Condition |
|---|---|
| `WORKTREE_REFUSAL_MAIN_WORKTREE` | `is_main` |
| `WORKTREE_REFUSAL_DIRECTORY_MISSING` | `WORKTREE_FLAG_MISSING` |
| `WORKTREE_REFUSAL_FACTS_UNKNOWN` | `facts_unavailable_reason` is not empty |
| `WORKTREE_REFUSAL_ACTIVE_SESSION` | The state is `WORKTREE_STATE_ACTIVE` |
| `WORKTREE_REFUSAL_PROCESS_RUNNING` | `WORKTREE_FLAG_BUSY` |
| `WORKTREE_REFUSAL_UNCOMMITTED_CHANGES` | `WORKTREE_FLAG_DIRTY` |
| `WORKTREE_REFUSAL_UNPUSHED_COMMITS` | `WORKTREE_FLAG_UNPUSHED`, and `merged_by` is not `WORKTREE_MERGED_BY_ANCESTOR` or `WORKTREE_MERGED_BY_TREE`: with these two rules the base branch has each change |
| `WORKTREE_REFUSAL_DETACHED_COMMITS` | `WORKTREE_FLAG_DETACHED_COMMITS` |
| `WORKTREE_REFUSAL_LOCKED` | `WORKTREE_FLAG_LOCKED` |
| `WORKTREE_REFUSAL_SUBMODULE` | `WORKTREE_FLAG_SUBMODULE` |
| `WORKTREE_REFUSAL_IGNORED_USER_FILES` | `WORKTREE_FLAG_IGNORED_USER_FILES` |
| `WORKTREE_REFUSAL_OTHER_FILE_SYSTEM` | `WORKTREE_FLAG_OTHER_FILE_SYSTEM` |
| `WORKTREE_REFUSAL_NOT_FINISHED` | The state is not `WORKTREE_STATE_FINISHED` |

A Worktree without a reason is finished and loses no work in a removal of its directory. Build output is not work: a build makes it again.

## Sizes

The daemon measures the size of each Worktree:

- `total_bytes` is the allocated size (`st_blocks` × 512) of each entry below the directory. The walk follows no symbolic link and does not go into a different file system. A hard link counts one time in the walk of a Project: the first Worktree that the walk measures gets its bytes. A nested Worktree is not in the sum of the Worktree around it.
- `build_output_bytes` is the part of `total_bytes` in build output. `build_outputs` has each build output directory with its bytes and the identifier of the rule that accepted it.
- A file whose blocks are shared with a different file (a reflink) counts in full. Thus the sum of the sizes is an upper bound of the space that a removal frees.

The daemon measures in the background and never on the path of a request:

- A Project whose sizes are absent or old gets a walk after the response to `Worktree.List`. With `wait_for_sizes`, the walk comes before the response.
- A size is old after the size lifetime, 1 hour in this version, or after the end of a Session in its Worktree.
- The daemon walks one Project at a time and has no timer for the sizes. A daemon that no client asks measures nothing.
- The daemon keeps the sizes in memory. A Worktree that the extension no longer gives loses its size.

## Requests

The Worktree service has the field numbers 900 to 999 in `Request.body` and in `Response.result`. It has no event in this version.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 900 | `Worktree.List` | `WorktreeListRequest` | `WorktreeListResponse` | Gives the Worktrees of one Project or of each Project, with their state, flags, size and refusals. |

- The Projects are in the sequence of the workspace tree. A Project without Worktrees has `unavailable_reason`.
- `free_bytes` and `file_system_bytes` are the space of the file system of the directory of the Project, as the user of the daemon can use it.

Errors of the Worktree service:

| Request | Condition | Code |
|---------|-----------|------|
| `WorktreeListRequest` | `project_id` is not empty, and no node has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `WorktreeListRequest` | `project_id` is the identifier of a Group. | `ERROR_CODE_FAILED` |

A failure of the extension is not an error of the request: it is the `unavailable_reason` of the Project.

## The method `worktree_facts`

The git extension `nomux.git` has the method `worktree_facts`. The argument is a `WorktreeFactsRequest`, and the result is a `WorktreeFactsResponse`. The daemon calls it one time for each Project of a `Worktree.List`.

- `repository` is the directory of the Project. The extension refuses a directory that is not in its approved directories with `ERROR_CODE_PERMISSION_DENIED`.
- The extension gives each worktree of the repository: the main worktree first, also the entry of a bare repository, then the linked worktrees.
- A linked worktree whose directory is not in the approved directories of the extension has `error`, and of its facts only the fields before `error`. These fields come from the common git directory, which is in the repository.
- The extension reads the repository and writes nothing: no object, no index, no lock and no reference. It starts no `git` process and opens no network connection. As `git status` and `git merge-tree` do, the status and the merge in memory can run a filter or a merge driver that the configuration of the repository names, for example the filter of Git LFS.
- `ignored_paths` has each ignored path of the worktree that git would list with `git status --ignored`: a directory whose content is all ignored and has no tracked file is one path that ends with `/`. The daemon classifies these paths with its build-output rules.
