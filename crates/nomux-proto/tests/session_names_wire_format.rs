//! Tests of the protobuf wire format of the session names and of the
//! activity of a Session: its fields of a Session, of `Session.Create` and
//! the request `Session.Rename`.
//!
//! Field numbers, wire types and capability flag names are permanent. Each
//! test has the expected bytes or values as literals, thus a changed schema
//! fails the test.

use nomux_proto::capabilities::{SESSION_ACTIVITY, SESSION_NAMES};
use nomux_proto::wire::v1::{
    Request, Response, Session, SessionCreateRequest, SessionCreateResponse, SessionRenameRequest,
    SessionRenameResponse, request, response,
};
use prost::Message;

#[test]
fn the_capability_flags_have_their_registered_names() {
    assert_eq!(SESSION_NAMES, "session_names");
    assert_eq!(SESSION_ACTIVITY, "session_activity");
}

#[test]
fn the_name_and_the_activity_of_a_session_are_fields_16_to_18() {
    let session = Session {
        id: "s".to_owned(),
        name: "api".to_owned(),
        last_attached_at: 5,
        attached_clients: 2,
        ..Session::default()
    };
    assert_eq!(
        session.encode_to_vec(),
        [
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x82, 0x01, 0x03, b'a', b'p', b'i', // field 16, bytes: name = "api"
            0x88, 0x01, 0x05, // field 17, varint: last_attached_at = 5
            0x90, 0x01, 0x02, // field 18, varint: attached_clients = 2
        ],
    );
}

#[test]
fn the_name_of_a_new_session_is_field_3_and_the_reuse_is_field_4() {
    let create = SessionCreateRequest {
        name: "api".to_owned(),
        reuse_existing: true,
        ..SessionCreateRequest::default()
    };
    assert_eq!(
        create.encode_to_vec(),
        [
            0x1a, 0x03, b'a', b'p', b'i', // field 3, bytes: name = "api"
            0x20, 0x01, // field 4, varint: reuse_existing = true
        ],
    );
    let created = SessionCreateResponse {
        session: None,
        created: true,
    };
    assert_eq!(created.encode_to_vec(), [0x10, 0x01]);
}

#[test]
fn the_rename_request_is_field_313() {
    let request = Request {
        id: 1,
        body: Some(request::Body::SessionRename(SessionRenameRequest {
            id: "s".to_owned(),
            name: "a".to_owned(),
        })),
    };
    assert_eq!(
        request.encode_to_vec(),
        [
            0x08, 0x01, // id = 1
            0xca, 0x13, 0x06, // field 313, bytes: session_rename
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x12, 0x01, b'a', // field 2, bytes: name = "a"
        ],
    );
    let response = Response {
        id: 1,
        result: Some(response::Result::SessionRename(SessionRenameResponse {
            session: None,
        })),
    };
    assert_eq!(response.encode_to_vec(), [0x08, 0x01, 0xca, 0x13, 0x00]);
}
