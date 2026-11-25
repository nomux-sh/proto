//! Tests of the protobuf wire format of the control messages.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::compatibility::EXIT_CODE_VERSION_INCOMPATIBLE;
use nomux_proto::wire::v1::{
    ClientKind, Envelope, Error, ErrorCode, Event, Hello, HostInfoRequest, HostInfoResponse,
    HostStopRequest, HostStopResponse, Request, Response, Session, SessionCreateRequest,
    SessionCreateResponse, SessionKillRequest, SessionKillResponse, SessionListRequest,
    SessionListResponse, SessionState, Welcome, envelope, request, response,
};
use prost::Message;

#[test]
fn hello_wire_bytes_are_stable() {
    let hello = Hello {
        version: 3,
        oldest: 2,
        capabilities: vec!["waits".to_owned()],
        client_kind: ClientKind::Mobile.into(),
        token: "t".to_owned(),
    };
    let expected = [
        0x08, 0x03, // field 1, varint: version = 3
        0x10, 0x02, // field 2, varint: oldest = 2
        0x1a, 0x05, b'w', b'a', b'i', b't', b's', // field 3, bytes: "waits"
        0x20, 0x03, // field 4, varint: client_kind = MOBILE
        0x2a, 0x01, b't', // field 5, bytes: token = "t"
    ];
    assert_eq!(hello.encode_to_vec(), expected);
}

#[test]
fn envelope_wire_bytes_are_stable() {
    let hello = Envelope {
        kind: Some(envelope::Kind::Request(Request {
            id: 1,
            body: Some(request::Body::Hello(Hello {
                version: 1,
                oldest: 1,
                capabilities: Vec::new(),
                client_kind: ClientKind::Unspecified.into(),
                token: String::new(),
            })),
        })),
    };
    let welcome = Envelope {
        kind: Some(envelope::Kind::Response(Response {
            id: 1,
            result: Some(response::Result::Welcome(Welcome {
                version: 1,
                oldest: 1,
                capabilities: vec!["waits".to_owned()],
                host_id: "h".to_owned(),
            })),
        })),
    };
    let error = Envelope {
        kind: Some(envelope::Kind::Response(Response {
            id: 7,
            result: Some(response::Result::Error(Error {
                code: ErrorCode::VersionIncompatible.into(),
                message: "m".to_owned(),
            })),
        })),
    };
    let event = Envelope {
        kind: Some(envelope::Kind::Event(Event { seq: 9 })),
    };

    let cases: [(Envelope, &[u8]); 4] = [
        (
            hello,
            &[
                0x0a, 0x09, // Envelope field 1, bytes: request
                0x08, 0x01, // Request field 1, varint: id = 1
                0xa2, 0x06, 0x04, // Request field 100, bytes: hello
                0x08, 0x01, 0x10, 0x01, // Hello: version = 1, oldest = 1
            ],
        ),
        (
            welcome,
            &[
                0x12, 0x13, // Envelope field 2, bytes: response
                0x08, 0x01, // Response field 1, varint: id = 1
                0xa2, 0x06, 0x0e, // Response field 100, bytes: welcome
                0x08, 0x01, // Welcome field 1, varint: version = 1
                0x10, 0x01, // Welcome field 2, varint: oldest = 1
                0x1a, 0x05, b'w', b'a', b'i', b't', b's', // Welcome field 3, bytes: "waits"
                0x22, 0x01, b'h', // Welcome field 4, bytes: host_id = "h"
            ],
        ),
        (
            error,
            &[
                0x12, 0x09, // Envelope field 2, bytes: response
                0x08, 0x07, // Response field 1, varint: id = 7
                0x12, 0x05, // Response field 2, bytes: error
                0x08, 0x05, // Error field 1, varint: code = VERSION_INCOMPATIBLE
                0x12, 0x01, b'm', // Error field 2, bytes: message = "m"
            ],
        ),
        (
            event,
            &[
                0x1a, 0x02, // Envelope field 3, bytes: event
                0x08, 0x09, // Event field 1, varint: seq = 9
            ],
        ),
    ];
    for (envelope, expected) in cases {
        assert_eq!(envelope.encode_to_vec(), expected, "{envelope:?}");
    }
}

#[test]
fn client_kind_values_are_stable() {
    assert_eq!(ClientKind::Unspecified as i32, 0);
    assert_eq!(ClientKind::Cli as i32, 1);
    assert_eq!(ClientKind::Desktop as i32, 2);
    assert_eq!(ClientKind::Mobile as i32, 3);
}

#[test]
fn error_code_values_are_the_exit_codes() {
    assert_eq!(ErrorCode::Unspecified as i32, 0);
    assert_eq!(ErrorCode::Failed as i32, 1);
    assert_eq!(ErrorCode::NotFound as i32, 2);
    assert_eq!(ErrorCode::Timeout as i32, 3);
    assert_eq!(ErrorCode::PermissionDenied as i32, 4);
    assert_eq!(
        ErrorCode::VersionIncompatible as i32,
        i32::from(EXIT_CODE_VERSION_INCOMPATIBLE)
    );
    assert_eq!(EXIT_CODE_VERSION_INCOMPATIBLE, 5);
}

#[test]
fn session_state_values_are_stable() {
    assert_eq!(SessionState::Unspecified as i32, 0);
    assert_eq!(SessionState::Running as i32, 1);
    assert_eq!(SessionState::Unreachable as i32, 2);
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

#[test]
fn host_service_wire_bytes_are_stable() {
    assert_eq!(
        request_bytes(request::Body::HostInfo(HostInfoRequest {})),
        [0x08, 0x01, 0xc2, 0x0c, 0x00], // id = 1; field 200, bytes: empty
    );
    assert_eq!(
        request_bytes(request::Body::HostStop(HostStopRequest {})),
        [0x08, 0x01, 0xca, 0x0c, 0x00], // id = 1; field 201, bytes: empty
    );
    assert_eq!(
        response_bytes(response::Result::HostInfo(HostInfoResponse {
            daemon_version: "v".to_owned(),
            daemon_pid: 7,
            session_count: 2,
        })),
        [
            0x08, 0x01, // id = 1
            0xc2, 0x0c, 0x07, // field 200, bytes: host_info
            0x0a, 0x01, b'v', // field 1, bytes: daemon_version = "v"
            0x10, 0x07, // field 2, varint: daemon_pid = 7
            0x18, 0x02, // field 3, varint: session_count = 2
        ],
    );
    assert_eq!(
        response_bytes(response::Result::HostStop(HostStopResponse {})),
        [0x08, 0x01, 0xca, 0x0c, 0x00],
    );
}

#[test]
fn session_service_wire_bytes_are_stable() {
    let session = Session {
        id: "s".to_owned(),
        state: SessionState::Unreachable.into(),
        pid: 9,
    };
    let session_bytes = [
        0x0a, 0x01, b's', // field 1, bytes: id = "s"
        0x10, 0x02, // field 2, varint: state = UNREACHABLE
        0x18, 0x09, // field 3, varint: pid = 9
    ];
    assert_eq!(session.encode_to_vec(), session_bytes);

    assert_eq!(
        request_bytes(request::Body::SessionCreate(SessionCreateRequest {
            command: vec!["a".to_owned(), "b".to_owned()],
            working_directory: "/".to_owned(),
        })),
        [
            0x08, 0x01, // id = 1
            0xe2, 0x12, 0x09, // field 300, bytes: session_create
            0x0a, 0x01, b'a', // field 1, bytes: command = "a"
            0x0a, 0x01, b'b', // field 1, bytes: command = "b"
            0x12, 0x01, b'/', // field 2, bytes: working_directory = "/"
        ],
    );
    assert_eq!(
        request_bytes(request::Body::SessionList(SessionListRequest {})),
        [0x08, 0x01, 0xea, 0x12, 0x00], // id = 1; field 301, bytes: empty
    );
    assert_eq!(
        request_bytes(request::Body::SessionKill(SessionKillRequest {
            id: "s".to_owned(),
        })),
        [
            0x08, 0x01, // id = 1
            0xf2, 0x12, 0x03, // field 302, bytes: session_kill
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
        ],
    );

    let mut created = vec![0x08, 0x01, 0xe2, 0x12, 0x09, 0x0a, 0x07];
    created.extend_from_slice(&session_bytes);
    assert_eq!(
        response_bytes(response::Result::SessionCreate(SessionCreateResponse {
            session: Some(session.clone()),
        })),
        created,
    );
    let mut listed = vec![0x08, 0x01, 0xea, 0x12, 0x09, 0x0a, 0x07];
    listed.extend_from_slice(&session_bytes);
    assert_eq!(
        response_bytes(response::Result::SessionList(SessionListResponse {
            sessions: vec![session],
        })),
        listed,
    );
    assert_eq!(
        response_bytes(response::Result::SessionKill(SessionKillResponse {})),
        [0x08, 0x01, 0xf2, 0x12, 0x00],
    );
}

#[test]
fn hello_debug_output_does_not_contain_the_token() {
    let envelope = Envelope {
        kind: Some(envelope::Kind::Request(Request {
            id: 1,
            body: Some(request::Body::Hello(Hello {
                version: 1,
                oldest: 1,
                capabilities: vec!["waits".to_owned()],
                client_kind: ClientKind::Cli.into(),
                token: "example-token".to_owned(),
            })),
        })),
    };
    let debug = format!("{envelope:?}");
    assert!(!debug.contains("example-token"), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
    assert!(debug.contains("waits"), "{debug}");
}
