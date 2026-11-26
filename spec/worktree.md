# Worktrees

**Status: DRAFT.** The rules can change in a minor version until version 1.0.0.

This document specifies the Worktrees of the Projects of a Host and the Worktree service of the control protocol: the states, the flags, the reasons why a removal is refused, the sizes, the plan and the clean, the trash place, the marks, the events, the requests, the errors, and the methods of the git extension that give the git facts and make the changes of git.
The schemas are in `proto/nomux/wire/v1/worktree.proto` and `proto/nomux/wire/v1/worktree_facts.proto`. The schemas are the authority for the fields of each message.

The daemon changes a repository only for a command of the user or for a rule that the user turned on in the settings of the daemon. By default the daemon only reports. The daemon has no git code: the git extension reads the repository and runs the `git` program of the user for each change of git.

## Terms

- **Worktree.** A directory that `git worktree list` gives for the repository of a Project: the main worktree or a linked worktree. A worktree that an agent tool made is an ordinary linked worktree.
- **Base branch.** The branch that the merge rules compare with. See [Merge rules](#merge-rules).
- **Build output.** A directory that a build or an install makes again, for example `target` or `node_modules`. The daemon decides with its build-output rules. The rules are data of the daemon and not a part of the protocol.
- **Trash place.** The directory `.nomux-trash` in the parent directory of a linked Worktree. A clean moves a finished Worktree there for the grace time. A restore moves it back.
- **Grace time.** The time that a Worktree stays in the trash place before the daemon removes it. It is a setting of the daemon.
- **Strict merge rule.** The branch is merged by `WORKTREE_MERGED_BY_ANCESTOR` or by `WORKTREE_MERGED_BY_TREE`: the base branch has each change of the branch. Only this rule lets a clean delete a branch. `WORKTREE_MERGED_BY_UPSTREAM_GONE` alone never deletes a branch.

## The Worktrees of a Project

- The Worktrees of a Project are the worktrees of the repository of the directory of the Project, wherever the directory of a Worktree is. Two Projects in the same repository have the same Worktrees.
- A Session belongs to the Worktree with the longest directory that contains its working directory. The comparison uses the rule of [workspace.md](workspace.md#projects-sessions-and-worktrees): the resolved working directory, complete path components. A Worktree can be below the main worktree; then a Session in it belongs to it and not to the main worktree.
- The daemon gets the git facts from the git extension `nomux.git`: [The method `worktree_facts`](#the-method-worktree_facts). The daemon has no git code. Without the extension, or when the approved directories of the extension do not contain the directory of the Project, the Project has no Worktree and `ProjectWorktrees.unavailable_reason` says why and what the user can do.
- The daemon asks the extension for each `Worktree.List`. It keeps no list of Worktrees. It keeps the sizes in memory ([Sizes](#sizes)), and the marks and the trash entries in its state directory.
- The daemon never fetches. `ProjectWorktrees.last_fetch_at_ms` tells the user how old the remote-tracking branches are. A client can run the fetch of the user when the user asks for it.
- `WorktreeInfo.tool` is the agent tool that probably made the Worktree. The daemon finds it with a table of directories and lock reasons, for example `.claude/worktrees/` for `claude-code`. It is a guess. The same rules apply to the Worktrees of each tool.

## States

A Worktree has one state. The daemon applies the rows in this sequence and gives the first that matches.

| State | Condition |
|---|---|
| `WORKTREE_STATE_ACTIVE` | A Session that is not ended has its working directory in the Worktree. A dormant Session counts. |
| `WORKTREE_STATE_FINISHED` | A merge rule matches the branch (`merged_by` is not `WORKTREE_MERGED_BY_UNSPECIFIED`), or the mark is `WORKTREE_MARK_FINISHED`. |
| `WORKTREE_STATE_STALE` | `last_activity_at_ms` is older than the stale time. The stale time is a setting of the daemon. Its default is 14 days. |
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
| `WORKTREE_FLAG_BUSY` | The daemon: a process of the Host has its current directory in the Worktree, or a build holds the lock file of a build output directory (for example `target/debug/.cargo-lock`). The daemon reads the current directory of each process where the operating system lets it, and tests each lock file with a lock that does not wait |
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
| `WORKTREE_REFUSAL_NO_TRASH_PLACE` | The parent directory of the directory is not writable |
| `WORKTREE_REFUSAL_IN_TRASH` | `trash_id` is not empty |
| `WORKTREE_REFUSAL_KEPT` | The mark is `WORKTREE_MARK_KEEP` |
| `WORKTREE_REFUSAL_NOT_FINISHED` | The state is not `WORKTREE_STATE_FINISHED` |

A Worktree without a reason is finished and loses no work in a removal of its directory. Build output is not work: a build makes it again.

## Sizes

The daemon measures the size of each Worktree:

- `total_bytes` is the allocated size (`st_blocks` × 512) of each entry below the directory. The walk follows no symbolic link and does not go into a different file system. A hard link counts one time in the walk of a Project: the first Worktree that the walk measures gets its bytes. A nested Worktree is not in the sum of the Worktree around it.
- `build_output_bytes` is the part of `total_bytes` in build output. `build_outputs` has each build output directory with its bytes and the identifier of the rule that accepted it.
- A file whose blocks are shared with a different file (a reflink) counts in full. Thus the sum of the sizes is an upper bound of the space that a removal frees.

The daemon measures in the background and never on the path of a request:

- A Project whose sizes are absent or old gets a walk after the response to `Worktree.List`. With `wait_for_sizes`, the walk comes before the response.
- A size is old after the size lifetime, or after the end of a Session in its Worktree, or after a clean of its Worktree. The size lifetime is a setting of the daemon. Its default is 1 hour.
- The daemon walks one Project at a time and has no timer for the sizes. A daemon that no client asks measures nothing.
- The daemon keeps the sizes in memory. A Worktree that the extension no longer gives loses its size.

## Marks

The user can set one mark on a Worktree with `Worktree.Mark`:

- `WORKTREE_MARK_FINISHED`: the Worktree is finished also when no merge rule matches. The branch stays unless the strict merge rule matches.
- `WORKTREE_MARK_KEEP`: each clean refuses the Worktree with `WORKTREE_REFUSAL_KEPT`, also the removal of its build output.

The daemon keeps the marks in its state directory. A mark goes away when a clean removes the Worktree or moves it to the trash place.

## Plan and clean

A clean has two requests. `Worktree.Plan` makes the list of the items of the clean and changes nothing. `Worktree.Clean` runs the items of one plan. A plan and a real clean use the same rules: a client shows the plan as the dry run.

### Items of a plan

`WorktreePlanRequest.action` selects the Worktrees and the steps. `worktree_paths` limits the plan to these Worktrees. A Worktree in the trash place gets no item.

| Action | Worktrees | Steps for each Worktree |
|---|---|---|
| `WORKTREE_CLEAN_ACTION_BUILD_OUTPUT` | Each linked Worktree. The main worktree only when `worktree_paths` has it. | One `WORKTREE_PLAN_STEP_BUILD_OUTPUT` for each build output directory |
| `WORKTREE_CLEAN_ACTION_STALE` | Each Worktree as above whose state is `WORKTREE_STATE_STALE` | As above |
| `WORKTREE_CLEAN_ACTION_FINISHED` | Each linked Worktree whose state is `WORKTREE_STATE_FINISHED`, and each Worktree that `worktree_paths` has | Each build output directory, then `WORKTREE_PLAN_STEP_TRASH`, or `WORKTREE_PLAN_STEP_DELETE` with `now`. Then the steps of `WORKTREE_CLEAN_ACTION_PRUNE` |
| `WORKTREE_CLEAN_ACTION_PRUNE` | Each linked Worktree whose directory is missing | One `WORKTREE_PLAN_STEP_PRUNE` for each record, or one `WORKTREE_PLAN_STEP_REPAIR` |

The refusals of an item:

- `WORKTREE_PLAN_STEP_BUILD_OUTPUT` of the actions `BUILD_OUTPUT` and `STALE`: `WORKTREE_REFUSAL_FACTS_UNKNOWN`, `WORKTREE_REFUSAL_ACTIVE_SESSION`, `WORKTREE_REFUSAL_PROCESS_RUNNING` and `WORKTREE_REFUSAL_KEPT`. Changes that are not committed do not refuse the removal of build output: git ignores all of it.
- `WORKTREE_PLAN_STEP_TRASH` and `WORKTREE_PLAN_STEP_DELETE`: each reason of [Refusals](#refusals), with these exceptions. `allow_unpushed` removes `WORKTREE_REFUSAL_UNPUSHED_COMMITS`. `include_ignored` removes `WORKTREE_REFUSAL_IGNORED_USER_FILES`. `now` removes `WORKTREE_REFUSAL_OTHER_FILE_SYSTEM` and `WORKTREE_REFUSAL_NO_TRASH_PLACE`. Nothing removes `WORKTREE_REFUSAL_DETACHED_COMMITS`: the user makes a branch first.
- In the action `FINISHED`, the build output steps of a Worktree get the refusals of its trash step. Thus a clean changes nothing in a Worktree that it refuses.
- `WORKTREE_PLAN_STEP_PRUNE`: `WORKTREE_REFUSAL_LOCKED` for a locked record. git does not prune a locked record.

A Session that is not ended but has no process (a dormant Session) does not refuse an item of a plan. The item has a warning: the restore of the Session will find no directory. A Session with a process refuses the item.

A build output directory is a directory that the build-output rules accept, that git ignores completely and that has no tracked file. Each other ignored path is a file of the user, for example `.env` or a local database. A clean never deletes a file of the user, except with `include_ignored` at the end of the grace time.

`WORKTREE_PLAN_STEP_REPAIR`: when a directory with the `.git` file of a missing record exists below the main worktree at the same relative path, for example after a move of the directory of the Project, the plan proposes `git worktree repair <path>` and has no prune step for that repository. `git worktree prune` removes each prunable record of the repository at one time, and it would remove the record of the moved directory.

With `delete_branch`, a trash step or a delete step has `deletes_branch` when its branch is merged by the strict merge rule. Otherwise `branch_kept_reason` says why the branch stays.

The daemon keeps each plan in memory for 15 minutes. A daemon that starts again has no plan.

### The clean

`Worktree.Clean` runs the items of the plan without a refusal in their sequence. The daemon runs one clean at a time. Before each item, the daemon makes the plan again with the same request:

- The item runs when the new plan has the same item (the same Worktree, step, path and `HEAD`) without a refusal.
- The item gets `WORKTREE_CLEAN_OUTCOME_REFUSED` when the new plan has it with a refusal, and `WORKTREE_CLEAN_OUTCOME_CHANGED` when the new plan does not have it.
- An item that the new plan has and the old plan did not have does not run.
- When a step of a Worktree fails, the later steps of that Worktree do not run.

The steps:

1. `WORKTREE_PLAN_STEP_BUILD_OUTPUT`: the daemon renames the directory to `<name>.nomux-deleting-<id>` in the same parent directory, then deletes it. The deletion follows no symbolic link and does not go into a different file system. The daemon writes the name of the renamed directory to its state directory before the rename. A deletion that a stop of the daemon interrupted continues at the start of the next clean.
2. `WORKTREE_PLAN_STEP_TRASH`: the daemon makes the trash place, and the git extension moves the directory there and locks it: [The changes of git](#the-changes-of-git). The daemon keeps the trash entry in its state directory.
3. `WORKTREE_PLAN_STEP_DELETE`: the git extension removes the Worktree, and with `deletes_branch` the branch.
4. `WORKTREE_PLAN_STEP_PRUNE`: the git extension prunes the records of the repository.

`freed_bytes` is the increase of the free space of the file systems of the Worktrees during the clean. `planned_bytes` is the sum of the bytes of the items that ran. A reflink and a hard link that a different Worktree shares free less than the plan counts.

### The trash place

- A Worktree in the trash place is locked with the reason `nomux-trash`. git refuses to remove it, and the branch cannot be checked out in a different Worktree.
- At the end of the grace time the daemon removes the Worktree with the git extension. With `deletes_branch`, the extension deletes the branch if the strict merge rule still matches and the tip of the branch is still `head`. Otherwise the branch stays. The grace time is a setting of the daemon. Its default is 24 hours.
- `Worktree.Restore` unlocks the Worktree and moves it back to `path`. Build output does not come back: a build makes it again.
- `Worktree.EmptyTrash` removes entries before the end of their grace time, with the same checks.
- The trash place is on the file system of the parent directory: the move is a rename. A Worktree whose directory is a mount point or on a different file system than its parent directory has no trash place. Only a delete step (`now`) removes it.
- The daemon has a timer only while the trash place has an entry.

### Rules of the settings

The settings of the daemon can turn on rules that make a clean without a command of the user. By default no rule is on. A rule uses the same plan with these limits: a dormant Session refuses the item, and `allow_unpushed`, `include_ignored`, `now` and `delete_branch` are false. The events of such a clean have `automatic`.

## Events

| Field number in `Event.body` | Event | Type for the filter | When |
|-----------------------------:|-------|---------------------|------|
| 900 | `WorktreeChanged` | `EVENT_TYPE_WORKTREE_CHANGED` | A clean deleted build output, moved a Worktree to the trash place, removed a Worktree or pruned records; a restore moved a Worktree back; the grace time of an entry ended; a mark changed. |
| 901 | `WorktreeDiskPressure` | `EVENT_TYPE_WORKTREE_DISK_PRESSURE` | The free space of a file system with Worktrees went to a different level. |

The daemon reads the free space of the file system of each Project when a client asks for `Worktree.List`, after a clean, and every 5 minutes while a client has a subscription to the events. A daemon without a subscription has no timer for it.

| Level | Free bytes |
|---|---|
| `WORKTREE_DISK_LEVEL_CRITICAL` | Below the critical level, a setting of the daemon. Its default is 3 GiB. |
| `WORKTREE_DISK_LEVEL_WARNING` | Below the warning level, a setting of the daemon. Its default is 10 GiB. |
| `WORKTREE_DISK_LEVEL_UNSPECIFIED` | Each other value |

- The daemon sends `WorktreeDiskPressure` when the level of a file system is different from the level of the last event of that file system. The first level of a file system after the start of the daemon is `WORKTREE_DISK_LEVEL_UNSPECIFIED`.
- `command` shows the plan of a clean of the finished Worktrees and changes nothing. The event deletes nothing.

## Requests

The Worktree service has the field numbers 900 to 999 in `Request.body` and in `Response.result`.

| Field number | Method | Request | Response | Function |
|-------------:|--------|---------|----------|----------|
| 900 | `Worktree.List` | `WorktreeListRequest` | `WorktreeListResponse` | Gives the Worktrees of one Project or of each Project, with their state, flags, size and refusals. |
| 901 | `Worktree.Plan` | `WorktreePlanRequest` | `WorktreePlanResponse` | Gives the items that a clean would run, and the identifier of the plan. Changes nothing. |
| 902 | `Worktree.Clean` | `WorktreeCleanRequest` | `WorktreeCleanResponse` | Runs the items of a plan. |
| 903 | `Worktree.Mark` | `WorktreeMarkRequest` | `WorktreeMarkResponse` | Sets or removes the mark of a Worktree. |
| 904 | `Worktree.Trash` | `WorktreeTrashRequest` | `WorktreeTrashResponse` | Gives the entries of the trash place. |
| 905 | `Worktree.Restore` | `WorktreeRestoreRequest` | `WorktreeRestoreResponse` | Moves a Worktree from the trash place back. |
| 906 | `Worktree.EmptyTrash` | `WorktreeEmptyTrashRequest` | `WorktreeEmptyTrashResponse` | Removes entries of the trash place now. |

- The Projects are in the sequence of the workspace tree. A Project without Worktrees has `unavailable_reason`.
- `free_bytes` and `file_system_bytes` are the space of the file system of the directory of the Project, as the user of the daemon can use it.

Errors of the Worktree service:

| Request | Condition | Code |
|---------|-----------|------|
| `WorktreeListRequest`, `WorktreePlanRequest`, `WorktreeTrashRequest` | `project_id` is not empty, and no node has the identifier. | `ERROR_CODE_NOT_FOUND` |
| `WorktreeListRequest`, `WorktreePlanRequest`, `WorktreeTrashRequest` | `project_id` is the identifier of a Group. | `ERROR_CODE_FAILED` |
| `WorktreePlanRequest` | A path of `worktree_paths` is no Worktree of the Projects. | `ERROR_CODE_NOT_FOUND` |
| `WorktreeCleanRequest` | The daemon has no plan with `plan_id`: it is older than 15 minutes, or the daemon started again. | `ERROR_CODE_NOT_FOUND` |
| `WorktreeCleanRequest` | A different clean runs. | `ERROR_CODE_FAILED` |
| `WorktreeMarkRequest` | `path` is no Worktree of a Project. | `ERROR_CODE_NOT_FOUND` |
| `WorktreeRestoreRequest` | No entry has `name`, or more than one entry has it as the last part of its path. | `ERROR_CODE_NOT_FOUND` |
| `WorktreeRestoreRequest` | `path` of the entry exists. | `ERROR_CODE_FAILED` |
| `WorktreeEmptyTrashRequest` | An identifier of `ids` is no entry. | `ERROR_CODE_NOT_FOUND` |

A failure of the extension in `Worktree.List` is not an error of the request: it is the `unavailable_reason` of the Project. In a clean, it is the outcome of an item.

## The method `worktree_facts`

The git extension `nomux.git` has the method `worktree_facts`. The argument is a `WorktreeFactsRequest`, and the result is a `WorktreeFactsResponse`. The daemon calls it one time for each Project of a `Worktree.List`.

- `repository` is the directory of the Project. The extension refuses a directory that is not in its approved directories with `ERROR_CODE_PERMISSION_DENIED`.
- The extension gives each worktree of the repository: the main worktree first, also the entry of a bare repository, then the linked worktrees.
- A linked worktree whose directory is not in the approved directories of the extension has `error`, and of its facts only the fields before `error`. These fields come from the common git directory, which is in the repository.
- The extension reads the repository and writes nothing: no object, no index, no lock and no reference. It starts no `git` process and opens no network connection. As `git status` and `git merge-tree` do, the status and the merge in memory can run a filter or a merge driver that the configuration of the repository names, for example the filter of Git LFS.
- `ignored_paths` has each ignored path of the worktree that git would list with `git status --ignored`: a directory whose content is all ignored and has no tracked file is one path that ends with `/`. The daemon classifies these paths with its build-output rules.
- `base_branch` is the base branch that the user set in the settings of the daemon. The extension uses it in place of the rule of [Merge rules](#merge-rules) when the branch exists: first as a remote-tracking branch, then as a local branch.

## The changes of git

The git extension makes each change of git with the `git` program of the user. The daemon calls these methods only for a clean, a restore and the end of a grace time. Each method checks the facts of the worktree itself before it changes something, thus a change is refused in two places: in the plan of the daemon and in the extension. No method passes `--force` to git.

| Method | Argument, result | What it does | Refuses |
|---|---|---|---|
| `worktree_move_to_trash` | `WorktreeMoveToTrashRequest`, `WorktreeMoveToTrashResponse` | `git worktree move <path> <trash_path>`, then `git worktree lock --reason nomux-trash <trash_path>` | The main worktree; a missing directory; facts that are not known; changes that are not committed; commits that are not pushed, unless `allow_unpushed` or the strict merge rule matches; detached commits; a lock; a submodule; an ignored path that is not in `allowed_ignored_paths`; a `trash_path` on a different file system |
| `worktree_move_from_trash` | `WorktreeMoveFromTrashRequest`, `WorktreeMoveFromTrashResponse` | `git worktree unlock <trash_path>`, then `git worktree move <trash_path> <path>` | A worktree that is not locked with the reason `nomux-trash`; an existing `path` (an error) |
| `worktree_delete` | `WorktreeDeleteRequest`, `WorktreeDeleteResponse` | `git worktree unlock` if the reason of the lock is `nomux-trash`, then `git worktree remove <path>`, then with `delete_branch` `git branch -D <branch>` | The checks of `worktree_move_to_trash`, except a lock with the reason `nomux-trash`. The branch stays when the strict merge rule does not match or when its tip is not `expected_head` |
| `worktree_prune` | `WorktreePruneRequest`, `WorktreePruneResponse` | `git worktree prune` | A set of prunable records that is different from `paths` |

- A refusal is not an error: the result has the refusals and a message. An error of `git` is an error of the call with `ERROR_CODE_FAILED` and the message of git.
- A `HEAD` that is not `expected_head` is an error of the call with `ERROR_CODE_FAILED`: the plan is old. Nothing changes.
- `worktree_delete` locks the worktree again with the reason `nomux-trash` when `git worktree remove` fails after the unlock.
- The extension refuses each path that is not in its approved directories with `ERROR_CODE_PERMISSION_DENIED`. The trash place is in the parent directory of a worktree: the user approves a directory that contains the worktrees and their parent directory.
