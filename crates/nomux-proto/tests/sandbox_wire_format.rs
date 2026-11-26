//! Tests of the protobuf wire format of the sandbox of a Session: its field
//! of a Session, of `Session.Create` and of `Host.Info`.
//!
//! Field numbers, wire types and capability flag names are permanent. Each
//! test has the expected bytes or values as literals, thus a changed schema
//! fails the test.

use nomux_proto::capabilities::SESSION_SANDBOX;
use nomux_proto::wire::v1::{
    HostInfoResponse, SandboxState, SandboxSupport, Session, SessionCreateRequest, SessionSandbox,
};
use prost::Message;

#[test]
fn the_capability_flag_has_its_registered_name() {
    assert_eq!(SESSION_SANDBOX, "session_sandbox");
}

#[test]
fn the_sandbox_of_a_session_is_field_19() {
    let session = Session {
        sandbox: Some(Box::new(SessionSandbox {
            profile: "p".to_owned(),
            state: SandboxState::Partial.into(),
            enforced: vec!["f".to_owned()],
            missing: vec!["u".to_owned()],
        })),
        ..Session::default()
    };
    assert_eq!(
        session.encode_to_vec(),
        [
            0x9a, 0x01, 0x0b, // field 19, bytes: sandbox
            0x0a, 0x01, b'p', // field 1, bytes: profile = "p"
            0x10, 0x02, // field 2, varint: state = SANDBOX_STATE_PARTIAL
            0x1a, 0x01, b'f', // field 3, bytes: enforced = ["f"]
            0x22, 0x01, b'u', // field 4, bytes: missing = ["u"]
        ],
    );
}

#[test]
fn the_profile_of_a_new_session_is_field_5() {
    let create = SessionCreateRequest {
        sandbox: "project".to_owned(),
        ..SessionCreateRequest::default()
    };
    assert_eq!(
        create.encode_to_vec(),
        [0x2a, 0x07, b'p', b'r', b'o', b'j', b'e', b'c', b't'],
    );
}

#[test]
fn the_sandbox_support_of_the_host_is_field_5() {
    let info = HostInfoResponse {
        sandbox: Some(SandboxSupport {
            enforced: vec!["f".to_owned()],
            missing: vec!["u".to_owned()],
            detail: "d".to_owned(),
        }),
        ..HostInfoResponse::default()
    };
    assert_eq!(
        info.encode_to_vec(),
        [
            0x2a, 0x09, // field 5, bytes: sandbox
            0x0a, 0x01, b'f', // field 1, bytes: enforced = ["f"]
            0x12, 0x01, b'u', // field 2, bytes: missing = ["u"]
            0x1a, 0x01, b'd', // field 3, bytes: detail = "d"
        ],
    );
}

#[test]
fn the_values_of_the_sandbox_state_are_fixed() {
    assert_eq!(SandboxState::Unspecified as i32, 0);
    assert_eq!(SandboxState::Enforced as i32, 1);
    assert_eq!(SandboxState::Partial as i32, 2);
    assert_eq!(SandboxState::Inactive as i32, 3);
}
