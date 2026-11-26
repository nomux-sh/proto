//! Tests of the protobuf wire format of the Worktree service and of the
//! payloads of the method `worktree_facts` of the git extension.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    BuildOutputSize, ProjectWorktrees, Request, Response, WorktreeFacts, WorktreeFactsRequest,
    WorktreeFactsResponse, WorktreeFlag, WorktreeInfo, WorktreeListRequest, WorktreeListResponse,
    WorktreeMergedBy, WorktreeRefusal, WorktreeSize, WorktreeState, request, response,
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
