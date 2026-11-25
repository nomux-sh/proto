//! Encode and decode tests for the generated control messages.

use nomux_proto::compat::VersionRange;
use nomux_proto::wire::v1::{
    ClientKind, Envelope, Error, ErrorCode, Hello, Request, Response, Welcome, envelope, request,
    response,
};
use prost::Message;

fn sample_hello() -> Hello {
    Hello {
        version: VersionRange::CURRENT.version,
        oldest: VersionRange::CURRENT.oldest,
        capabilities: vec!["waits".to_owned(), "pane_state_events".to_owned()],
        client_kind: ClientKind::Cli.into(),
        token: "example-token".to_owned(),
    }
}

#[test]
fn hello_roundtrip() {
    let hello = sample_hello();
    let bytes = hello.encode_to_vec();
    assert_eq!(Hello::decode(bytes.as_slice()), Ok(hello));
}

#[test]
fn hello_debug_output_does_not_contain_the_token() {
    let envelope = Envelope {
        kind: Some(envelope::Kind::Request(Request {
            id: 1,
            body: Some(request::Body::Hello(sample_hello())),
        })),
    };
    let debug = format!("{envelope:?}");
    assert!(!debug.contains("example-token"), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
    assert!(debug.contains("pane_state_events"), "{debug}");
}

#[test]
fn hello_wire_bytes_are_stable() {
    // Field numbers and wire types are permanent. This test fails if a field
    // of `Hello` gets a different number or type.
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
fn client_kind_values_are_stable() {
    assert_eq!(ClientKind::Unspecified as i32, 0);
    assert_eq!(ClientKind::Cli as i32, 1);
    assert_eq!(ClientKind::Desktop as i32, 2);
    assert_eq!(ClientKind::Mobile as i32, 3);
}

#[test]
fn unknown_client_kind_decodes_and_is_not_a_known_value() {
    // A newer peer can send a value that this version does not know.
    let hello = Hello {
        client_kind: 99,
        ..sample_hello()
    };
    let decoded = Hello::decode(hello.encode_to_vec().as_slice()).expect("the message is valid");
    assert_eq!(decoded.client_kind, 99);
    assert!(ClientKind::try_from(decoded.client_kind).is_err());
}

#[test]
fn handshake_request_and_response_roundtrip() {
    let request = Envelope {
        kind: Some(envelope::Kind::Request(Request {
            id: 1,
            body: Some(request::Body::Hello(sample_hello())),
        })),
    };
    let bytes = request.encode_length_delimited_to_vec();
    assert_eq!(
        Envelope::decode_length_delimited(bytes.as_slice()),
        Ok(request)
    );

    let response = Envelope {
        kind: Some(envelope::Kind::Response(Response {
            id: 1,
            result: Some(response::Result::Welcome(Welcome {
                version: 1,
                oldest: 1,
                capabilities: Vec::new(),
                host_id: "example-host".to_owned(),
            })),
        })),
    };
    let bytes = response.encode_length_delimited_to_vec();
    assert_eq!(
        Envelope::decode_length_delimited(bytes.as_slice()),
        Ok(response)
    );
}

#[test]
fn version_incompatible_error_has_the_exit_code_value() {
    assert_eq!(
        ErrorCode::VersionIncompatible as i32,
        i32::from(nomux_proto::compat::EXIT_CODE_VERSION_INCOMPATIBLE)
    );
    assert_eq!(ErrorCode::PermissionDenied as i32, 4);

    let response = Response {
        id: 7,
        result: Some(response::Result::Error(Error {
            code: ErrorCode::VersionIncompatible.into(),
            message: "no common protocol version".to_owned(),
        })),
    };
    let bytes = response.encode_to_vec();
    assert_eq!(Response::decode(bytes.as_slice()), Ok(response));
}
