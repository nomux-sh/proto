//! Tests of the protobuf wire format of the Ext service.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    ExtCallError, ExtCallErrorCode, ExtCallRequest, ExtCallResponse, ExtInspectRequest,
    ExtInspectResponse, ExtInstallRequest, ExtInstallResponse, ExtListRequest, ExtListResponse,
    ExtRemoveRequest, ExtRemoveResponse, ExtSetEnabledRequest, ExtSetEnabledResponse, Extension,
    ExtensionPermissions, ExtensionState, Request, Response, ext_call_response, request, response,
};
use prost::Message;

const PERMISSIONS_BYTES: [u8; 13] = [
    0x0a, 0x01, b'/', // field 1, bytes: fs = "/"
    0x12, 0x01, b'p', // field 2, bytes: process = "p"
    0x18, 0x01, // field 3, varint: network = true
    0x22, 0x03, b's', b':', b'r', // field 4, bytes: scopes = "s:r"
];

fn permissions() -> ExtensionPermissions {
    ExtensionPermissions {
        fs: vec!["/".to_owned()],
        process: vec!["p".to_owned()],
        network: true,
        scopes: vec!["s:r".to_owned()],
    }
}

fn extension() -> Extension {
    Extension {
        id: "a.b".to_owned(),
        name: "n".to_owned(),
        version: "1".to_owned(),
        state: ExtensionState::Disabled.into(),
        state_reason: "r".to_owned(),
        granted: Some(permissions()),
    }
}

fn extension_bytes() -> Vec<u8> {
    let mut bytes = vec![
        0x0a, 0x03, b'a', b'.', b'b', // field 1, bytes: id = "a.b"
        0x12, 0x01, b'n', // field 2, bytes: name = "n"
        0x1a, 0x01, b'1', // field 3, bytes: version = "1"
        0x20, 0x05, // field 4, varint: state = DISABLED
        0x2a, 0x01, b'r', // field 5, bytes: state_reason = "r"
        0x32, 0x0d, // field 6, bytes: granted
    ];
    bytes.extend_from_slice(&PERMISSIONS_BYTES);
    bytes
}

/// The bytes of a request or a response with the `id` 1 and one field of
/// the Ext service. `tag` is the first byte of the two bytes of the field
/// number: the second byte is 0x19 for each field from 400 to 415.
fn in_ext_field(tag: u8, message: &[u8]) -> Vec<u8> {
    let length = u8::try_from(message.len()).expect("a short message");
    let mut bytes = vec![0x08, 0x01, tag, 0x19, length];
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

#[test]
fn extension_state_values_are_stable() {
    assert_eq!(ExtensionState::Unspecified as i32, 0);
    assert_eq!(ExtensionState::Stopped as i32, 1);
    assert_eq!(ExtensionState::Starting as i32, 2);
    assert_eq!(ExtensionState::Running as i32, 3);
    assert_eq!(ExtensionState::Failed as i32, 4);
    assert_eq!(ExtensionState::Disabled as i32, 5);
}

#[test]
fn call_error_code_values_are_the_codes_of_the_extension_protocol() {
    assert_eq!(ExtCallErrorCode::Unspecified as i32, 0);
    assert_eq!(ExtCallErrorCode::Failed as i32, 1);
    assert_eq!(ExtCallErrorCode::NotFound as i32, 2);
    assert_eq!(ExtCallErrorCode::Timeout as i32, 3);
    assert_eq!(ExtCallErrorCode::PermissionDenied as i32, 4);
    assert_eq!(ExtCallErrorCode::VersionIncompatible as i32, 5);
    assert_eq!(ExtCallErrorCode::InvalidArgument as i32, 6);
    assert_eq!(ExtCallErrorCode::Cancelled as i32, 7);
}

#[test]
fn extension_wire_bytes_are_stable() {
    assert_eq!(permissions().encode_to_vec(), PERMISSIONS_BYTES);
    assert_eq!(extension().encode_to_vec(), extension_bytes());
}

#[test]
fn list_and_install_wire_bytes_are_stable() {
    assert_eq!(
        request_bytes(request::Body::ExtList(ExtListRequest {})),
        in_ext_field(0x82, &[]), // field 400
    );
    let mut listed = vec![0x0a, 0x1f]; // field 1, bytes: extensions
    listed.extend_from_slice(&extension_bytes());
    assert_eq!(
        response_bytes(response::Result::ExtList(ExtListResponse {
            extensions: vec![extension()],
        })),
        in_ext_field(0x82, &listed),
    );

    let directory = [0x0a, 0x02, b'/', b'd']; // field 1, bytes: directory = "/d"
    assert_eq!(
        request_bytes(request::Body::ExtInspect(ExtInspectRequest {
            directory: "/d".to_owned(),
        })),
        in_ext_field(0x8a, &directory), // field 401
    );
    let mut inspected = vec![
        0x0a, 0x03, b'a', b'.', b'b', // field 1, bytes: id = "a.b"
        0x12, 0x01, b'n', // field 2, bytes: name = "n"
        0x1a, 0x01, b'1', // field 3, bytes: version = "1"
        0x22, 0x0d, // field 4, bytes: requested
    ];
    inspected.extend_from_slice(&PERMISSIONS_BYTES);
    assert_eq!(
        response_bytes(response::Result::ExtInspect(ExtInspectResponse {
            id: "a.b".to_owned(),
            name: "n".to_owned(),
            version: "1".to_owned(),
            requested: Some(permissions()),
        })),
        in_ext_field(0x8a, &inspected),
    );

    let mut install = directory.to_vec();
    install.extend_from_slice(&[0x12, 0x0d]); // field 2, bytes: granted
    install.extend_from_slice(&PERMISSIONS_BYTES);
    assert_eq!(
        request_bytes(request::Body::ExtInstall(ExtInstallRequest {
            directory: "/d".to_owned(),
            granted: Some(permissions()),
        })),
        in_ext_field(0x92, &install), // field 402
    );
    let mut installed = vec![0x0a, 0x1f]; // field 1, bytes: extension
    installed.extend_from_slice(&extension_bytes());
    assert_eq!(
        response_bytes(response::Result::ExtInstall(ExtInstallResponse {
            extension: Some(extension()),
        })),
        in_ext_field(0x92, &installed),
    );
}

#[test]
fn remove_and_set_enabled_wire_bytes_are_stable() {
    let id = [0x0a, 0x03, b'a', b'.', b'b']; // field 1, bytes: id = "a.b"
    assert_eq!(
        request_bytes(request::Body::ExtRemove(ExtRemoveRequest {
            id: "a.b".to_owned(),
        })),
        in_ext_field(0x9a, &id), // field 403
    );
    assert_eq!(
        response_bytes(response::Result::ExtRemove(ExtRemoveResponse {})),
        in_ext_field(0x9a, &[]),
    );

    let mut enable = id.to_vec();
    enable.extend_from_slice(&[0x10, 0x01]); // field 2, varint: enabled = true
    assert_eq!(
        request_bytes(request::Body::ExtSetEnabled(ExtSetEnabledRequest {
            id: "a.b".to_owned(),
            enabled: true,
        })),
        in_ext_field(0xa2, &enable), // field 404
    );
    let mut changed = vec![0x0a, 0x1f]; // field 1, bytes: extension
    changed.extend_from_slice(&extension_bytes());
    assert_eq!(
        response_bytes(response::Result::ExtSetEnabled(ExtSetEnabledResponse {
            extension: Some(extension()),
        })),
        in_ext_field(0xa2, &changed),
    );
}

#[test]
fn call_wire_bytes_are_stable() {
    assert_eq!(
        request_bytes(request::Body::ExtCall(ExtCallRequest {
            ext: "a.b".to_owned(),
            method: "m".to_owned(),
            payload: vec![0x01],
        })),
        in_ext_field(
            0xaa, // field 405
            &[
                0x0a, 0x03, b'a', b'.', b'b', // field 1, bytes: ext = "a.b"
                0x12, 0x01, b'm', // field 2, bytes: method = "m"
                0x1a, 0x01, 0x01, // field 3, bytes: payload
            ],
        ),
    );
    assert_eq!(
        response_bytes(response::Result::ExtCall(ExtCallResponse {
            answer: Some(ext_call_response::Answer::Payload(vec![0x02])),
        })),
        in_ext_field(0xaa, &[0x0a, 0x01, 0x02]), // field 1, bytes: payload
    );
    // An empty result is on the wire: the client must see that the
    // extension answered.
    assert_eq!(
        response_bytes(response::Result::ExtCall(ExtCallResponse {
            answer: Some(ext_call_response::Answer::Payload(Vec::new())),
        })),
        in_ext_field(0xaa, &[0x0a, 0x00]),
    );
    assert_eq!(
        response_bytes(response::Result::ExtCall(ExtCallResponse {
            answer: Some(ext_call_response::Answer::Error(ExtCallError {
                code: ExtCallErrorCode::InvalidArgument.into(),
                message: "m".to_owned(),
            })),
        })),
        in_ext_field(
            0xaa,
            &[
                0x12, 0x05, // field 2, bytes: error
                0x08, 0x06, // field 1, varint: code = INVALID_ARGUMENT
                0x12, 0x01, b'm', // field 2, bytes: message = "m"
            ],
        ),
    );
}
