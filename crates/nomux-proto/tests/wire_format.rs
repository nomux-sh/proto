//! Tests of the protobuf wire format of the control messages.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::capabilities::OUTPUT_HISTORY;
use nomux_proto::compatibility::EXIT_CODE_VERSION_INCOMPATIBLE;
use nomux_proto::wire::v1::{
    ClientKind, Envelope, Error, ErrorCode, Event, Hello, HostInfoRequest, HostInfoResponse,
    HostStopRequest, HostStopResponse, MemoryMeasure, Request, Response, RestorePolicy, Session,
    SessionAttachRequest, SessionAttachResponse, SessionCaptureRequest, SessionCaptureResponse,
    SessionCreateRequest, SessionCreateResponse, SessionDetachRequest, SessionDetachResponse,
    SessionExit, SessionInputRequest, SessionInputResponse, SessionKillRequest,
    SessionKillResponse, SessionListRequest, SessionListResponse, SessionMemory,
    SessionMemoryRequest, SessionMemoryResponse, SessionResizeRequest, SessionResizeResponse,
    SessionSetRestorePolicyRequest, SessionSetRestorePolicyResponse, SessionStartRequest,
    SessionStartResponse, SessionState, Welcome, envelope, request, response, session_exit,
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
        kind: Some(envelope::Kind::Event(Event {
            seq: 9,
            ..Event::default()
        })),
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
    assert_eq!(SessionState::Dormant as i32, 3);
    assert_eq!(SessionState::Ended as i32, 4);
    assert_eq!(SessionState::Damaged as i32, 5);
}

#[test]
fn restore_policy_values_are_stable() {
    assert_eq!(RestorePolicy::Unspecified as i32, 0);
    assert_eq!(RestorePolicy::Auto as i32, 1);
    assert_eq!(RestorePolicy::OnFocus as i32, 2);
    assert_eq!(RestorePolicy::Manual as i32, 3);
}

#[test]
fn memory_measure_values_are_stable() {
    assert_eq!(MemoryMeasure::Unspecified as i32, 0);
    assert_eq!(MemoryMeasure::Proportional as i32, 1);
    assert_eq!(MemoryMeasure::Resident as i32, 2);
}

#[test]
fn the_name_of_a_capability_flag_is_the_name_of_the_registry() {
    assert_eq!(OUTPUT_HISTORY, "output_history");
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
            memory_limit: None,
            sandbox: None,
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
        ..Session::default()
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
            ..SessionCreateRequest::default()
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
        request_bytes(request::Body::SessionList(SessionListRequest {
            include_ended: false,
        })),
        [0x08, 0x01, 0xea, 0x12, 0x00], // id = 1; field 301, bytes: empty
    );
    assert_eq!(
        request_bytes(request::Body::SessionList(SessionListRequest {
            include_ended: true,
        })),
        [
            0x08, 0x01, // id = 1
            0xea, 0x12, 0x02, // field 301, bytes: session_list
            0x08, 0x01, // field 1, varint: include_ended = true
        ],
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
            created: false,
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
fn terminal_requests_of_the_session_service_have_stable_wire_bytes() {
    assert_eq!(
        request_bytes(request::Body::SessionAttach(SessionAttachRequest {
            id: "s".to_owned(),
            rows: 24,
            columns: 80,
            epoch: 1,
            from_seq: 300,
        })),
        [
            0x08, 0x01, // id = 1
            0xfa, 0x12, 0x0c, // field 303, bytes: session_attach
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0x18, // field 2, varint: rows = 24
            0x18, 0x50, // field 3, varint: columns = 80
            0x20, 0x01, // field 4, varint: epoch = 1
            0x28, 0xac, 0x02, // field 5, varint: from_seq = 300
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionAttach(SessionAttachResponse {
            handle: 1,
            epoch: 2,
        })),
        [
            0x08, 0x01, // id = 1
            0xfa, 0x12, 0x04, // field 303, bytes: session_attach
            0x08, 0x01, // field 1, varint: handle = 1
            0x10, 0x02, // field 2, varint: epoch = 2
        ],
    );

    assert_eq!(
        request_bytes(request::Body::SessionDetach(SessionDetachRequest {
            id: "s".to_owned(),
        })),
        [
            0x08, 0x01, // id = 1
            0x82, 0x13, 0x03, // field 304, bytes: session_detach
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionDetach(SessionDetachResponse {
            detached: 2,
        })),
        [
            0x08, 0x01, // id = 1
            0x82, 0x13, 0x02, // field 304, bytes: session_detach
            0x08, 0x02, // field 1, varint: detached = 2
        ],
    );

    assert_eq!(
        request_bytes(request::Body::SessionInput(SessionInputRequest {
            id: "s".to_owned(),
            data: vec![0x03],
        })),
        [
            0x08, 0x01, // id = 1
            0x8a, 0x13, 0x06, // field 305, bytes: session_input
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x12, 0x01, 0x03, // field 2, bytes: data
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionInput(SessionInputResponse {})),
        [0x08, 0x01, 0x8a, 0x13, 0x00],
    );

    assert_eq!(
        request_bytes(request::Body::SessionResize(SessionResizeRequest {
            id: "s".to_owned(),
            rows: 24,
            columns: 80,
        })),
        [
            0x08, 0x01, // id = 1
            0x92, 0x13, 0x07, // field 306, bytes: session_resize
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0x18, // field 2, varint: rows = 24
            0x18, 0x50, // field 3, varint: columns = 80
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionResize(SessionResizeResponse {})),
        [0x08, 0x01, 0x92, 0x13, 0x00],
    );
}

#[test]
fn the_description_of_a_session_has_stable_wire_bytes() {
    let session = Session {
        id: "s".to_owned(),
        state: SessionState::Ended.into(),
        pid: 0,
        command: vec!["a".to_owned(), "b".to_owned()],
        working_directory: "/".to_owned(),
        rows: 24,
        columns: 80,
        started_at: 5,
        epoch: 2,
        restore_policy: RestorePolicy::Manual.into(),
        exit: Some(SessionExit {
            ended_at: 6,
            status: Some(session_exit::Status::Signal(9)),
        }),
        damage: "d".to_owned(),
        resolved_working_directory: "/r".to_owned(),
        hibernation: None,
        pinned: false,
        name: String::new(),
        last_attached_at: 0,
        attached_clients: 0,
        sandbox: None,
    };
    assert_eq!(
        session.encode_to_vec(),
        [
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0x04, // field 2, varint: state = ENDED
            0x22, 0x01, b'a', // field 4, bytes: command = "a"
            0x22, 0x01, b'b', // field 4, bytes: command = "b"
            0x2a, 0x01, b'/', // field 5, bytes: working_directory = "/"
            0x30, 0x18, // field 6, varint: rows = 24
            0x38, 0x50, // field 7, varint: columns = 80
            0x40, 0x05, // field 8, varint: started_at = 5
            0x48, 0x02, // field 9, varint: epoch = 2
            0x50, 0x03, // field 10, varint: restore_policy = MANUAL
            0x5a, 0x04, // field 11, bytes: exit
            0x08, 0x06, // SessionExit field 1, varint: ended_at = 6
            0x18, 0x09, // SessionExit field 3, varint: signal = 9
            0x62, 0x01, b'd', // field 12, bytes: damage = "d"
            0x6a, 0x02, b'/', b'r', // field 13, bytes: resolved_working_directory = "/r"
        ],
    );
    let exited = SessionExit {
        ended_at: 0,
        status: Some(session_exit::Status::ExitCode(0)),
    };
    // An exit code of 0 is on the wire: it is not the same as no status.
    assert_eq!(exited.encode_to_vec(), [0x10, 0x00]);
}

#[test]
fn restore_requests_of_the_session_service_have_stable_wire_bytes() {
    assert_eq!(
        request_bytes(request::Body::SessionStart(SessionStartRequest {
            id: "s".to_owned(),
        })),
        [
            0x08, 0x01, // id = 1
            0x9a, 0x13, 0x03, // field 307, bytes: session_start
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionStart(SessionStartResponse {
            session: None,
        })),
        [0x08, 0x01, 0x9a, 0x13, 0x00],
    );

    assert_eq!(
        request_bytes(request::Body::SessionCapture(SessionCaptureRequest {
            id: "s".to_owned(),
            epoch: 2,
            before_seq: Some(0),
            max_bytes: 300,
        })),
        [
            0x08, 0x01, // id = 1
            0xa2, 0x13, 0x0a, // field 308, bytes: session_capture
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0x02, // field 2, varint: epoch = 2
            0x18, 0x00, // field 3, varint: before_seq = 0, which is not "absent"
            0x20, 0xac, 0x02, // field 4, varint: max_bytes = 300
        ],
    );
    assert_eq!(
        request_bytes(request::Body::SessionCapture(SessionCaptureRequest {
            id: String::new(),
            epoch: 0,
            before_seq: None,
            max_bytes: 0,
        })),
        [0x08, 0x01, 0xa2, 0x13, 0x00],
    );
    assert_eq!(
        response_bytes(response::Result::SessionCapture(SessionCaptureResponse {
            epoch: 2,
            seq: 300,
            data: vec![0x1b],
        })),
        [
            0x08, 0x01, // id = 1
            0xa2, 0x13, 0x08, // field 308, bytes: session_capture
            0x08, 0x02, // field 1, varint: epoch = 2
            0x10, 0xac, 0x02, // field 2, varint: seq = 300
            0x1a, 0x01, 0x1b, // field 3, bytes: data
        ],
    );

    assert_eq!(
        request_bytes(request::Body::SessionSetRestorePolicy(
            SessionSetRestorePolicyRequest {
                id: "s".to_owned(),
                policy: RestorePolicy::Auto.into(),
            }
        )),
        [
            0x08, 0x01, // id = 1
            0xaa, 0x13, 0x05, // field 309, bytes: session_set_restore_policy
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0x01, // field 2, varint: policy = AUTO
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionSetRestorePolicy(
            SessionSetRestorePolicyResponse {}
        )),
        [0x08, 0x01, 0xaa, 0x13, 0x00],
    );
}

#[test]
fn the_memory_request_of_the_session_service_has_stable_wire_bytes() {
    assert_eq!(
        request_bytes(request::Body::SessionMemory(SessionMemoryRequest {})),
        [0x08, 0x01, 0xb2, 0x13, 0x00], // id = 1; field 310, bytes: empty
    );
    assert_eq!(
        response_bytes(response::Result::SessionMemory(SessionMemoryResponse {
            sessions: vec![SessionMemory {
                id: "s".to_owned(),
                command_bytes: 300,
                command_processes: 2,
                holder_bytes: 9,
                measure: MemoryMeasure::Resident.into(),
                limit_bytes: 0,
            }],
        })),
        [
            0x08, 0x01, // id = 1
            0xb2, 0x13, 0x0e, // field 310, bytes: session_memory
            0x0a, 0x0c, // field 1, bytes: sessions
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0xac, 0x02, // field 2, varint: command_bytes = 300
            0x18, 0x02, // field 3, varint: command_processes = 2
            0x20, 0x09, // field 4, varint: holder_bytes = 9
            0x28, 0x02, // field 5, varint: measure = RESIDENT
        ],
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
