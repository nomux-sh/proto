//! Tests of the protobuf wire format of the Workspace service and of its
//! event, and of the limits of the workspace tree.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    Envelope, Event, EventType, MAX_ENVELOPE_LENGTH, Request, Response, WorkspaceAddProjectRequest,
    WorkspaceAddProjectResponse, WorkspaceChanged, WorkspaceCreateGroupRequest,
    WorkspaceCreateGroupResponse, WorkspaceGroup, WorkspaceMoveRequest, WorkspaceMoveResponse,
    WorkspaceNode, WorkspaceNodeAdded, WorkspaceNodeMoved, WorkspaceNodeRemoved,
    WorkspaceNodeRenamed, WorkspaceProject, WorkspaceRemoveGroupRequest,
    WorkspaceRemoveGroupResponse, WorkspaceRemoveProjectRequest, WorkspaceRemoveProjectResponse,
    WorkspaceRenameRequest, WorkspaceRenameResponse, WorkspaceTreeRequest, WorkspaceTreeResponse,
    envelope, event, request, response, workspace_changed, workspace_node,
};
use prost::Message;

/// The two bytes of the tags of the fields 800 to 806 with a length.
const WORKSPACE_TREE: [u8; 2] = [0x82, 0x32];
const WORKSPACE_ADD_PROJECT: [u8; 2] = [0x8a, 0x32];
const WORKSPACE_REMOVE_PROJECT: [u8; 2] = [0x92, 0x32];
const WORKSPACE_CREATE_GROUP: [u8; 2] = [0x9a, 0x32];
const WORKSPACE_REMOVE_GROUP: [u8; 2] = [0xa2, 0x32];
const WORKSPACE_RENAME: [u8; 2] = [0xaa, 0x32];
const WORKSPACE_MOVE: [u8; 2] = [0xb2, 0x32];

/// The limits of spec/workspace.md.
const MAX_NODES: usize = 800;
const MAX_ID_BYTES: usize = 32;
const MAX_NAME_BYTES: usize = 128;
const MAX_DIRECTORY_BYTES: usize = 1024;

/// The bytes of a message with the `id` 1 and one field with a length.
fn with_id(tag: [u8; 2], message: &[u8]) -> Vec<u8> {
    let length = u8::try_from(message.len()).expect("a short message");
    let mut bytes = vec![0x08, 0x01];
    bytes.extend(tag);
    bytes.push(length);
    bytes.extend_from_slice(message);
    bytes
}

fn request_bytes(body: request::Body) -> Vec<u8> {
    Request {
        id: 1,
        body: Some(body),
    }
    .encode_to_vec()
}

fn response_bytes(result: response::Result) -> Vec<u8> {
    Response {
        id: 1,
        result: Some(result),
    }
    .encode_to_vec()
}

fn project(id: &str, parent_id: &str, name: &str, directory: &str) -> WorkspaceNode {
    WorkspaceNode {
        id: id.to_owned(),
        parent_id: parent_id.to_owned(),
        name: name.to_owned(),
        kind: Some(workspace_node::Kind::Project(WorkspaceProject {
            directory: directory.to_owned(),
        })),
    }
}

fn group(id: &str, parent_id: &str, name: &str) -> WorkspaceNode {
    WorkspaceNode {
        id: id.to_owned(),
        parent_id: parent_id.to_owned(),
        name: name.to_owned(),
        kind: Some(workspace_node::Kind::Group(WorkspaceGroup {})),
    }
}

#[test]
fn the_event_type_of_the_workspace_is_4() {
    assert_eq!(EventType::WorkspaceChanged as i32, 4);
}

#[test]
fn a_node_has_its_kind_in_field_4_or_5() {
    let expected = [
        0x0a, 0x01, b'p', // field 1, bytes: id = "p"
        0x12, 0x01, b'g', // field 2, bytes: parent_id = "g"
        0x1a, 0x01, b'n', // field 3, bytes: name = "n"
        0x2a, 0x04, 0x0a, 0x02, b'/', b'r', // field 5, bytes: project, directory = "/r"
    ];
    assert_eq!(project("p", "g", "n", "/r").encode_to_vec(), expected);

    let expected = [
        0x0a, 0x01, b'g', // field 1, bytes: id = "g"
        0x1a, 0x01, b'n', // field 3, bytes: name = "n"
        0x22, 0x00, // field 4, bytes: group
    ];
    assert_eq!(group("g", "", "n").encode_to_vec(), expected);
}

#[test]
fn workspace_requests_use_the_field_numbers_from_800() {
    let tree = request::Body::WorkspaceTree(WorkspaceTreeRequest {});
    assert_eq!(request_bytes(tree), with_id(WORKSPACE_TREE, &[]));

    let add = request::Body::WorkspaceAddProject(WorkspaceAddProjectRequest {
        directory: "/r".to_owned(),
        name: "n".to_owned(),
        parent_id: "g".to_owned(),
        index: Some(2),
    });
    let fields = [
        0x0a, 0x02, b'/', b'r', // field 1, bytes: directory = "/r"
        0x12, 0x01, b'n', // field 2, bytes: name = "n"
        0x1a, 0x01, b'g', // field 3, bytes: parent_id = "g"
        0x20, 0x02, // field 4, varint: index = 2
    ];
    assert_eq!(request_bytes(add), with_id(WORKSPACE_ADD_PROJECT, &fields));

    let remove =
        request::Body::WorkspaceRemoveProject(WorkspaceRemoveProjectRequest { id: "p".to_owned() });
    assert_eq!(
        request_bytes(remove),
        with_id(WORKSPACE_REMOVE_PROJECT, &[0x0a, 0x01, b'p'])
    );

    let create = request::Body::WorkspaceCreateGroup(WorkspaceCreateGroupRequest {
        name: "n".to_owned(),
        parent_id: "g".to_owned(),
        index: Some(1),
    });
    let fields = [0x0a, 0x01, b'n', 0x12, 0x01, b'g', 0x18, 0x01];
    assert_eq!(
        request_bytes(create),
        with_id(WORKSPACE_CREATE_GROUP, &fields)
    );

    let remove =
        request::Body::WorkspaceRemoveGroup(WorkspaceRemoveGroupRequest { id: "g".to_owned() });
    assert_eq!(
        request_bytes(remove),
        with_id(WORKSPACE_REMOVE_GROUP, &[0x0a, 0x01, b'g'])
    );

    let rename = request::Body::WorkspaceRename(WorkspaceRenameRequest {
        id: "p".to_owned(),
        name: "n".to_owned(),
    });
    assert_eq!(
        request_bytes(rename),
        with_id(WORKSPACE_RENAME, &[0x0a, 0x01, b'p', 0x12, 0x01, b'n'])
    );

    let to_the_end = request::Body::WorkspaceMove(WorkspaceMoveRequest {
        id: "p".to_owned(),
        parent_id: String::new(),
        index: None,
    });
    assert_eq!(
        request_bytes(to_the_end),
        with_id(WORKSPACE_MOVE, &[0x0a, 0x01, b'p'])
    );
}

#[test]
fn an_index_of_0_is_sent_and_differs_from_an_absent_index() {
    let first = WorkspaceMoveRequest {
        id: "p".to_owned(),
        parent_id: "g".to_owned(),
        index: Some(0),
    };
    let expected = [
        0x0a, 0x01, b'p', // field 1, bytes: id = "p"
        0x12, 0x01, b'g', // field 2, bytes: parent_id = "g"
        0x18, 0x00, // field 3, varint: index = 0, present
    ];
    assert_eq!(first.encode_to_vec(), expected);

    let decoded = WorkspaceMoveRequest::decode(&expected[..]).expect("valid bytes");
    assert_eq!(decoded.index, Some(0));
    let decoded = WorkspaceMoveRequest::decode(&expected[..6]).expect("valid bytes");
    assert_eq!(decoded.index, None);
}

#[test]
fn workspace_responses_use_the_field_numbers_from_800() {
    let tree = response::Result::WorkspaceTree(WorkspaceTreeResponse {
        revision: 7,
        nodes: vec![group("g", "", "n")],
    });
    let fields = [
        0x08, 0x07, // field 1, varint: revision = 7
        0x12, 0x08, 0x0a, 0x01, b'g', 0x1a, 0x01, b'n', 0x22, 0x00, // field 2: one node
    ];
    assert_eq!(response_bytes(tree), with_id(WORKSPACE_TREE, &fields));

    let node = group("g", "", "n").encode_to_vec();
    let mut fields = vec![0x08, 0x07, 0x12, 0x08];
    fields.extend(&node);

    let add = response::Result::WorkspaceAddProject(WorkspaceAddProjectResponse {
        revision: 7,
        node: Some(group("g", "", "n")),
    });
    assert_eq!(response_bytes(add), with_id(WORKSPACE_ADD_PROJECT, &fields));
    let create = response::Result::WorkspaceCreateGroup(WorkspaceCreateGroupResponse {
        revision: 7,
        node: Some(group("g", "", "n")),
    });
    assert_eq!(
        response_bytes(create),
        with_id(WORKSPACE_CREATE_GROUP, &fields)
    );

    let revision = [0x08, 0x07];
    let results = [
        (
            response::Result::WorkspaceRemoveProject(WorkspaceRemoveProjectResponse {
                revision: 7,
            }),
            WORKSPACE_REMOVE_PROJECT,
        ),
        (
            response::Result::WorkspaceRemoveGroup(WorkspaceRemoveGroupResponse { revision: 7 }),
            WORKSPACE_REMOVE_GROUP,
        ),
        (
            response::Result::WorkspaceRename(WorkspaceRenameResponse { revision: 7 }),
            WORKSPACE_RENAME,
        ),
        (
            response::Result::WorkspaceMove(WorkspaceMoveResponse { revision: 7 }),
            WORKSPACE_MOVE,
        ),
    ];
    for (result, tag) in results {
        assert_eq!(response_bytes(result), with_id(tag, &revision));
    }
}

fn changed_event(change: workspace_changed::Change) -> Vec<u8> {
    Event {
        seq: 1,
        at_ms: 2,
        body: Some(event::Body::WorkspaceChanged(WorkspaceChanged {
            revision: 3,
            change: Some(change),
        })),
    }
    .encode_to_vec()
}

/// The bytes of an event with `seq` 1, `at_ms` 2 and a `WorkspaceChanged`
/// with the revision 3 and one change.
fn expected_event(change_tag: u8, change: &[u8]) -> Vec<u8> {
    let change_length = u8::try_from(change.len()).expect("a short change");
    let mut body = vec![0x08, 0x03, change_tag, change_length];
    body.extend_from_slice(change);
    let body_length = u8::try_from(body.len()).expect("a short body");
    let mut bytes = vec![0x08, 0x01, 0x10, 0x02];
    bytes.extend(WORKSPACE_TREE); // field 800 of `Event.body`
    bytes.push(body_length);
    bytes.extend(body);
    bytes
}

#[test]
fn a_workspace_change_is_field_800_of_an_event() {
    let added = workspace_changed::Change::Added(WorkspaceNodeAdded {
        node: Some(group("g", "", "n")),
        index: 1,
    });
    let change = [
        0x0a, 0x08, 0x0a, 0x01, b'g', 0x1a, 0x01, b'n', 0x22, 0x00, // field 1: node
        0x10, 0x01, // field 2, varint: index = 1
    ];
    assert_eq!(changed_event(added), expected_event(0x12, &change));

    let renamed = workspace_changed::Change::Renamed(WorkspaceNodeRenamed {
        id: "g".to_owned(),
        name: "m".to_owned(),
    });
    let change = [0x0a, 0x01, b'g', 0x12, 0x01, b'm'];
    assert_eq!(changed_event(renamed), expected_event(0x1a, &change));

    let moved = workspace_changed::Change::Moved(WorkspaceNodeMoved {
        id: "p".to_owned(),
        parent_id: "g".to_owned(),
        index: 2,
    });
    let change = [0x0a, 0x01, b'p', 0x12, 0x01, b'g', 0x18, 0x02];
    assert_eq!(changed_event(moved), expected_event(0x22, &change));

    let removed = workspace_changed::Change::Removed(WorkspaceNodeRemoved { id: "g".to_owned() });
    assert_eq!(
        changed_event(removed),
        expected_event(0x2a, &[0x0a, 0x01, b'g'])
    );
}

/// The limit of the node count holds because the largest tree goes into one
/// `Envelope`. If a limit of spec/workspace.md grows, this test fails.
#[test]
fn the_largest_tree_goes_into_one_envelope() {
    let id = |number: usize| format!("{number:0>width$}", width = MAX_ID_BYTES);
    let name = "n".repeat(MAX_NAME_BYTES);
    let directory = format!("/{}", "d".repeat(MAX_DIRECTORY_BYTES - 1));
    let nodes = (0..MAX_NODES)
        .map(|number| project(&id(number), &id(MAX_NODES + number), &name, &directory))
        .collect();
    let envelope = Envelope {
        kind: Some(envelope::Kind::Response(Response {
            id: u64::MAX,
            result: Some(response::Result::WorkspaceTree(WorkspaceTreeResponse {
                revision: u64::MAX,
                nodes,
            })),
        })),
    };

    assert!(envelope.encoded_len() <= MAX_ENVELOPE_LENGTH);
}
