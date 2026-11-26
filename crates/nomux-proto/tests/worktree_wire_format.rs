//! Tests of the protobuf wire format of the Worktree service and of the
//! payloads of the method `worktree_facts` of the git extension.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    BuildOutputSize, Event, EventType, ProjectWorktrees, Request, Response, WorktreeChange,
    WorktreeChanged, WorktreeCleanAction, WorktreeCleanOutcome, WorktreeCleanRequest,
    WorktreeDeleteRequest, WorktreeDiskLevel, WorktreeDiskPressure, WorktreeEmptyTrashRequest,
    WorktreeFacts, WorktreeFactsRequest, WorktreeFactsResponse, WorktreeFlag, WorktreeInfo,
    WorktreeListRequest, WorktreeListResponse, WorktreeMark, WorktreeMarkRequest, WorktreeMergedBy,
    WorktreeMoveFromTrashRequest, WorktreeMoveToTrashRequest, WorktreeMoveToTrashResponse,
    WorktreePlanItem, WorktreePlanRequest, WorktreePlanStep, WorktreePruneRequest, WorktreeRefusal,
    WorktreeRestoreRequest, WorktreeSize, WorktreeState, WorktreeTrashRequest, event, request,
    response,
};
use prost::Message;

/// The two bytes of the tag of the field 900 with a length.
const WORKTREE_LIST: [u8; 2] = [0xa2, 0x38];

/// The bytes of a message with the `id` 1 and one field with a length.
fn with_id(tag: [u8; 2], message: &[u8]) -> Vec<u8> {
    let length = u8::try_from(message.len()).expect("a short message");
    let mut bytes = vec![0x08, 0x01];
    bytes.extend(tag);
    bytes.push(length);
    bytes.extend_from_slice(message);
    bytes
}

#[test]
fn enum_values_are_stable() {
    let states = [
        (WorktreeState::Unspecified, 0),
        (WorktreeState::Active, 1),
        (WorktreeState::Finished, 2),
        (WorktreeState::Stale, 3),
        (WorktreeState::Idle, 4),
    ];
    for (state, value) in states {
        assert_eq!(state as i32, value);
    }
    let flags = [
        (WorktreeFlag::Unspecified, 0),
        (WorktreeFlag::Dirty, 1),
        (WorktreeFlag::Unpushed, 2),
        (WorktreeFlag::DetachedCommits, 3),
        (WorktreeFlag::Locked, 4),
        (WorktreeFlag::Submodule, 5),
        (WorktreeFlag::IgnoredUserFiles, 6),
        (WorktreeFlag::Busy, 7),
        (WorktreeFlag::Missing, 8),
        (WorktreeFlag::OtherFileSystem, 9),
        (WorktreeFlag::UpstreamGone, 10),
    ];
    for (flag, value) in flags {
        assert_eq!(flag as i32, value);
    }
    let rules = [
        (WorktreeMergedBy::Unspecified, 0),
        (WorktreeMergedBy::Ancestor, 1),
        (WorktreeMergedBy::Tree, 2),
        (WorktreeMergedBy::UpstreamGone, 3),
    ];
    for (rule, value) in rules {
        assert_eq!(rule as i32, value);
    }
    let refusals = [
        (WorktreeRefusal::Unspecified, 0),
        (WorktreeRefusal::MainWorktree, 1),
        (WorktreeRefusal::DirectoryMissing, 2),
        (WorktreeRefusal::ActiveSession, 3),
        (WorktreeRefusal::ProcessRunning, 4),
        (WorktreeRefusal::UncommittedChanges, 5),
        (WorktreeRefusal::UnpushedCommits, 6),
        (WorktreeRefusal::DetachedCommits, 7),
        (WorktreeRefusal::Locked, 8),
        (WorktreeRefusal::Submodule, 9),
        (WorktreeRefusal::IgnoredUserFiles, 10),
        (WorktreeRefusal::OtherFileSystem, 11),
        (WorktreeRefusal::NotFinished, 12),
        (WorktreeRefusal::FactsUnknown, 13),
        (WorktreeRefusal::Kept, 14),
        (WorktreeRefusal::NoTrashPlace, 15),
        (WorktreeRefusal::InTrash, 16),
    ];
    for (refusal, value) in refusals {
        assert_eq!(refusal as i32, value);
    }
}

#[test]
fn the_list_request_is_field_900() {
    let list = Request {
        id: 1,
        body: Some(request::Body::WorktreeList(WorktreeListRequest {
            project_id: "p".to_owned(),
            wait_for_sizes: true,
        })),
    };
    let fields = [
        0x0a, 0x01, b'p', // field 1, bytes: project_id = "p"
        0x10, 0x01, // field 2, varint: wait_for_sizes = true
    ];
    assert_eq!(list.encode_to_vec(), with_id(WORKTREE_LIST, &fields));
}

#[test]
fn the_list_response_is_field_900_with_the_fields_of_a_worktree() {
    let worktree = WorktreeInfo {
        path: "/w".to_owned(),
        branch: "b".to_owned(),
        head: "h".to_owned(),
        is_main: false,
        is_bare: false,
        state: WorktreeState::Finished.into(),
        flags: vec![WorktreeFlag::UpstreamGone.into()],
        session_ids: vec!["s".to_owned()],
        size: Some(WorktreeSize {
            total_bytes: 9,
            build_output_bytes: 8,
            measured_at_ms: 7,
            is_old: true,
            build_outputs: vec![BuildOutputSize {
                path: "t".to_owned(),
                rule: "r".to_owned(),
                bytes: 8,
            }],
        }),
        last_activity_at_ms: 6,
        merged_by: WorktreeMergedBy::UpstreamGone.into(),
        unpushed_commits: 2,
        refusals: vec![WorktreeRefusal::IgnoredUserFiles.into()],
        ignored_user_paths: vec![".env".to_owned()],
        locked_reason: "r".to_owned(),
        facts_unavailable_reason: "x".to_owned(),
        tool: String::new(),
        mark: WorktreeMark::Unspecified.into(),
        trash_id: String::new(),
    };
    let worktree_bytes = [
        0x0a, 0x02, b'/', b'w', // field 1, bytes: path = "/w"
        0x12, 0x01, b'b', // field 2, bytes: branch = "b"
        0x1a, 0x01, b'h', // field 3, bytes: head = "h"
        0x30, 0x02, // field 6, varint: state = FINISHED
        0x3a, 0x01, 0x0a, // field 7, packed: flags = [UPSTREAM_GONE]
        0x42, 0x01, b's', // field 8, bytes: session_ids = ["s"]
        0x4a, 0x12, // field 9: size
        0x08, 0x09, 0x10, 0x08, 0x18, 0x07, 0x20, 0x01, // the sizes, the time, is_old
        0x2a, 0x08, // field 5 of the size: one build output
        0x0a, 0x01, b't', 0x12, 0x01, b'r', 0x18, 0x08, // path, rule, bytes
        0x50, 0x06, // field 10, varint: last_activity_at_ms = 6
        0x58, 0x03, // field 11, varint: merged_by = UPSTREAM_GONE
        0x60, 0x02, // field 12, varint: unpushed_commits = 2
        0x6a, 0x01, 0x0a, // field 13, packed: refusals = [IGNORED_USER_FILES]
        0x72, 0x04, b'.', b'e', b'n', b'v', // field 14, bytes: ignored_user_paths
        0x7a, 0x01, b'r', // field 15, bytes: locked_reason = "r"
        0x82, 0x01, 0x01, b'x', // field 16, bytes: facts_unavailable_reason = "x"
    ];
    assert_eq!(worktree.encode_to_vec(), worktree_bytes);

    let project = ProjectWorktrees {
        project_id: "p".to_owned(),
        unavailable_reason: "u".to_owned(),
        base_branch: "m".to_owned(),
        last_fetch_at_ms: 5,
        free_bytes: 4,
        file_system_bytes: 3,
        worktrees: Vec::new(),
    };
    let project_bytes = [
        0x0a, 0x01, b'p', // field 1, bytes: project_id = "p"
        0x12, 0x01, b'u', // field 2, bytes: unavailable_reason = "u"
        0x1a, 0x01, b'm', // field 3, bytes: base_branch = "m"
        0x20, 0x05, // field 4, varint: last_fetch_at_ms = 5
        0x28, 0x04, // field 5, varint: free_bytes = 4
        0x30, 0x03, // field 6, varint: file_system_bytes = 3
    ];
    assert_eq!(project.encode_to_vec(), project_bytes);

    let list = Response {
        id: 1,
        result: Some(response::Result::WorktreeList(WorktreeListResponse {
            projects: vec![project],
        })),
    };
    let mut fields = vec![0x0a, 0x0f]; // field 1: one project
    fields.extend(project_bytes);
    assert_eq!(list.encode_to_vec(), with_id(WORKTREE_LIST, &fields));
}

#[test]
fn the_payloads_of_the_method_worktree_facts_have_the_expected_bytes() {
    let request = WorktreeFactsRequest {
        repository: "/r".to_owned(),
        base_branch: String::new(),
    };
    assert_eq!(request.encode_to_vec(), [0x0a, 0x02, b'/', b'r']);

    let facts = WorktreeFacts {
        path: "/w".to_owned(),
        branch: "b".to_owned(),
        head: "h".to_owned(),
        is_main: true,
        is_bare: true,
        is_locked: true,
        locked_reason: "r".to_owned(),
        is_missing: true,
        has_submodules: true,
        last_activity_at_ms: 6,
        error: "e".to_owned(),
        is_dirty: true,
        unpushed_commits: 3,
        has_detached_commits: true,
        upstream_gone: true,
        merged_by: WorktreeMergedBy::Tree.into(),
        ignored_paths: vec!["t/".to_owned()],
        ignored_paths_truncated: true,
    };
    let facts_bytes = [
        0x0a, 0x02, b'/', b'w', // field 1, bytes: path = "/w"
        0x12, 0x01, b'b', // field 2, bytes: branch = "b"
        0x1a, 0x01, b'h', // field 3, bytes: head = "h"
        0x20, 0x01, // field 4, varint: is_main = true
        0x28, 0x01, // field 5, varint: is_bare = true
        0x30, 0x01, // field 6, varint: is_locked = true
        0x3a, 0x01, b'r', // field 7, bytes: locked_reason = "r"
        0x40, 0x01, // field 8, varint: is_missing = true
        0x48, 0x01, // field 9, varint: has_submodules = true
        0x50, 0x06, // field 10, varint: last_activity_at_ms = 6
        0x5a, 0x01, b'e', // field 11, bytes: error = "e"
        0x60, 0x01, // field 12, varint: is_dirty = true
        0x68, 0x03, // field 13, varint: unpushed_commits = 3
        0x70, 0x01, // field 14, varint: has_detached_commits = true
        0x78, 0x01, // field 15, varint: upstream_gone = true
        0x80, 0x01, 0x02, // field 16, varint: merged_by = TREE
        0x8a, 0x01, 0x02, b't', b'/', // field 17, bytes: ignored_paths = ["t/"]
        0x90, 0x01, 0x01, // field 18, varint: ignored_paths_truncated = true
    ];
    assert_eq!(facts.encode_to_vec(), facts_bytes);

    let response = WorktreeFactsResponse {
        base_branch: "m".to_owned(),
        last_fetch_at_ms: 4,
        worktrees: vec![facts],
    };
    let mut expected = vec![
        0x0a, 0x01, b'm', // field 1, bytes: base_branch = "m"
        0x10, 0x04, // field 2, varint: last_fetch_at_ms = 4
        0x1a, 0x2f, // field 3: one worktree
    ];
    expected.extend(facts_bytes);
    assert_eq!(response.encode_to_vec(), expected);
}

#[test]
fn the_enum_values_of_the_clean_are_stable() {
    let values = [
        (WorktreeMark::Unspecified as i32, 0),
        (WorktreeMark::Finished as i32, 1),
        (WorktreeMark::Keep as i32, 2),
        (WorktreeCleanAction::Unspecified as i32, 0),
        (WorktreeCleanAction::BuildOutput as i32, 1),
        (WorktreeCleanAction::Finished as i32, 2),
        (WorktreeCleanAction::Stale as i32, 3),
        (WorktreeCleanAction::Prune as i32, 4),
        (WorktreePlanStep::Unspecified as i32, 0),
        (WorktreePlanStep::BuildOutput as i32, 1),
        (WorktreePlanStep::Trash as i32, 2),
        (WorktreePlanStep::Delete as i32, 3),
        (WorktreePlanStep::Prune as i32, 4),
        (WorktreePlanStep::Repair as i32, 5),
        (WorktreeCleanOutcome::Unspecified as i32, 0),
        (WorktreeCleanOutcome::Done as i32, 1),
        (WorktreeCleanOutcome::Changed as i32, 2),
        (WorktreeCleanOutcome::Refused as i32, 3),
        (WorktreeCleanOutcome::Failed as i32, 4),
        (WorktreeChange::Unspecified as i32, 0),
        (WorktreeChange::BuildOutputRemoved as i32, 1),
        (WorktreeChange::Trashed as i32, 2),
        (WorktreeChange::Restored as i32, 3),
        (WorktreeChange::Removed as i32, 4),
        (WorktreeChange::Pruned as i32, 5),
        (WorktreeChange::Marked as i32, 6),
        (WorktreeDiskLevel::Unspecified as i32, 0),
        (WorktreeDiskLevel::Warning as i32, 1),
        (WorktreeDiskLevel::Critical as i32, 2),
        (EventType::WorktreeChanged as i32, 9),
        (EventType::WorktreeDiskPressure as i32, 10),
    ];
    for (index, (value, expected)) in values.into_iter().enumerate() {
        assert_eq!(value, expected, "value {index}");
    }
}

#[test]
fn the_requests_of_the_clean_are_fields_901_to_906() {
    let bodies = [
        (
            request::Body::WorktreePlan(WorktreePlanRequest::default()),
            [0xaa, 0x38],
        ),
        (
            request::Body::WorktreeClean(WorktreeCleanRequest::default()),
            [0xb2, 0x38],
        ),
        (
            request::Body::WorktreeMark(WorktreeMarkRequest::default()),
            [0xba, 0x38],
        ),
        (
            request::Body::WorktreeTrash(WorktreeTrashRequest::default()),
            [0xc2, 0x38],
        ),
        (
            request::Body::WorktreeRestore(WorktreeRestoreRequest::default()),
            [0xca, 0x38],
        ),
        (
            request::Body::WorktreeEmptyTrash(WorktreeEmptyTrashRequest::default()),
            [0xd2, 0x38],
        ),
    ];
    for (body, tag) in bodies {
        let request = Request {
            id: 1,
            body: Some(body),
        };
        assert_eq!(request.encode_to_vec(), with_id(tag, &[]));
    }
}

#[test]
fn the_plan_request_and_an_item_have_the_expected_bytes() {
    let request = WorktreePlanRequest {
        project_id: "p".to_owned(),
        action: WorktreeCleanAction::Finished.into(),
        worktree_paths: vec!["/w".to_owned()],
        delete_branch: true,
        allow_unpushed: true,
        include_ignored: true,
        now: true,
    };
    let expected = [
        0x0a, 0x01, b'p', // field 1, bytes: project_id = "p"
        0x10, 0x02, // field 2, varint: action = FINISHED
        0x1a, 0x02, b'/', b'w', // field 3, bytes: worktree_paths = ["/w"]
        0x20, 0x01, 0x28, 0x01, 0x30, 0x01, 0x38, 0x01, // fields 4 to 7: true
    ];
    assert_eq!(request.encode_to_vec(), expected);

    let item = WorktreePlanItem {
        project_id: "p".to_owned(),
        worktree_path: "/w".to_owned(),
        step: WorktreePlanStep::Trash.into(),
        path: "/t".to_owned(),
        rule: "r".to_owned(),
        bytes: 9,
        branch: "b".to_owned(),
        head: "h".to_owned(),
        refusals: vec![WorktreeRefusal::Kept.into()],
        warnings: vec!["w".to_owned()],
        deletes_branch: true,
        branch_kept_reason: "k".to_owned(),
        ignored_user_paths: vec!["e".to_owned()],
    };
    let expected = [
        0x0a, 0x01, b'p', // field 1, bytes: project_id = "p"
        0x12, 0x02, b'/', b'w', // field 2, bytes: worktree_path = "/w"
        0x18, 0x02, // field 3, varint: step = TRASH
        0x22, 0x02, b'/', b't', // field 4, bytes: path = "/t"
        0x2a, 0x01, b'r', // field 5, bytes: rule = "r"
        0x30, 0x09, // field 6, varint: bytes = 9
        0x3a, 0x01, b'b', // field 7, bytes: branch = "b"
        0x42, 0x01, b'h', // field 8, bytes: head = "h"
        0x4a, 0x01, 0x0e, // field 9, packed: refusals = [KEPT]
        0x52, 0x01, b'w', // field 10, bytes: warnings = ["w"]
        0x58, 0x01, // field 11, varint: deletes_branch = true
        0x62, 0x01, b'k', // field 12, bytes: branch_kept_reason = "k"
        0x6a, 0x01, b'e', // field 13, bytes: ignored_user_paths = ["e"]
    ];
    assert_eq!(item.encode_to_vec(), expected);
}

#[test]
fn the_events_of_the_worktrees_are_fields_900_and_901_of_the_event() {
    let changed = Event {
        seq: 1,
        at_ms: 2,
        body: Some(event::Body::WorktreeChanged(WorktreeChanged {
            change: WorktreeChange::Trashed.into(),
            ..WorktreeChanged::default()
        })),
    };
    assert_eq!(
        changed.encode_to_vec(),
        [0x08, 0x01, 0x10, 0x02, 0xa2, 0x38, 0x02, 0x18, 0x02]
    );
    let pressure = Event {
        seq: 1,
        at_ms: 2,
        body: Some(event::Body::WorktreeDiskPressure(WorktreeDiskPressure {
            level: WorktreeDiskLevel::Critical.into(),
            ..WorktreeDiskPressure::default()
        })),
    };
    assert_eq!(
        pressure.encode_to_vec(),
        [0x08, 0x01, 0x10, 0x02, 0xaa, 0x38, 0x02, 0x10, 0x02]
    );
}

#[test]
fn the_payloads_of_the_changes_of_git_have_the_expected_bytes() {
    let facts = WorktreeFactsRequest {
        repository: "/r".to_owned(),
        base_branch: "m".to_owned(),
    };
    assert_eq!(
        facts.encode_to_vec(),
        [0x0a, 0x02, b'/', b'r', 0x12, 0x01, b'm']
    );
    let to_trash = WorktreeMoveToTrashRequest {
        repository: "/r".to_owned(),
        path: "/w".to_owned(),
        trash_path: "/t".to_owned(),
        expected_head: "h".to_owned(),
        allowed_ignored_paths: vec!["i".to_owned()],
        allow_unpushed: true,
        base_branch: "m".to_owned(),
    };
    assert_eq!(
        to_trash.encode_to_vec(),
        [
            0x0a, 0x02, b'/', b'r', 0x12, 0x02, b'/', b'w', 0x1a, 0x02, b'/', b't', 0x22, 0x01,
            b'h', 0x2a, 0x01, b'i', 0x30, 0x01, 0x3a, 0x01, b'm',
        ]
    );
    let refused = WorktreeMoveToTrashResponse {
        refusals: vec![WorktreeRefusal::UncommittedChanges.into()],
        message: "m".to_owned(),
    };
    assert_eq!(
        refused.encode_to_vec(),
        [0x0a, 0x01, 0x05, 0x12, 0x01, b'm']
    );
    let from_trash = WorktreeMoveFromTrashRequest {
        repository: "/r".to_owned(),
        trash_path: "/t".to_owned(),
        path: "/w".to_owned(),
    };
    assert_eq!(
        from_trash.encode_to_vec(),
        [
            0x0a, 0x02, b'/', b'r', 0x12, 0x02, b'/', b't', 0x1a, 0x02, b'/', b'w'
        ]
    );
    let delete = WorktreeDeleteRequest {
        repository: "/r".to_owned(),
        path: "/w".to_owned(),
        expected_head: "h".to_owned(),
        allowed_ignored_paths: vec!["i".to_owned()],
        allow_unpushed: true,
        delete_branch: true,
        base_branch: "m".to_owned(),
    };
    assert_eq!(
        delete.encode_to_vec(),
        [
            0x0a, 0x02, b'/', b'r', 0x12, 0x02, b'/', b'w', 0x1a, 0x01, b'h', 0x22, 0x01, b'i',
            0x28, 0x01, 0x30, 0x01, 0x3a, 0x01, b'm',
        ]
    );
    let prune = WorktreePruneRequest {
        repository: "/r".to_owned(),
        paths: vec!["/w".to_owned()],
    };
    assert_eq!(
        prune.encode_to_vec(),
        [0x0a, 0x02, b'/', b'r', 0x12, 0x02, b'/', b'w']
    );
}
