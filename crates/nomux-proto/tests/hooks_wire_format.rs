//! Tests of the protobuf wire format of the Hooks service, of the details of
//! an Agent run, of the events of the extensions and of the payloads of the
//! adapter extensions.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    AgentDetails, AgentReport, AgentRun, AgentState, ChildWorkChange, ChildWorkReport, Event,
    ExtensionEvent, HookAdapter, HookAnswer, HookEvent, HookEventRequest, HookEventResponse,
    HookInstallPlan, HookPlatform, HooksAdaptersRequest, HooksAdaptersResponse, HooksPlanRequest,
    HooksPlanResponse, NeedsYouKind, Request, Response, SettingsEntry, SettingsPath, event,
    request, response,
};
use prost::Message;

/// The two bytes of the tags of the fields 700, 701 and 702 with a length.
const HOOK_EVENT: [u8; 2] = [0xe2, 0x2b];
const HOOKS_PLAN: [u8; 2] = [0xea, 0x2b];
const HOOKS_ADAPTERS: [u8; 2] = [0xf2, 0x2b];

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

#[test]
fn enum_values_are_stable() {
    assert_eq!(HookPlatform::Unspecified as i32, 0);
    assert_eq!(HookPlatform::Linux as i32, 1);
    assert_eq!(HookPlatform::Macos as i32, 2);
    assert_eq!(ChildWorkChange::Unspecified as i32, 0);
    assert_eq!(ChildWorkChange::Start as i32, 1);
    assert_eq!(ChildWorkChange::Update as i32, 2);
    assert_eq!(ChildWorkChange::End as i32, 3);
}

#[test]
fn hooks_requests_use_the_field_numbers_from_700() {
    let hook = request::Body::HookEvent(HookEventRequest {
        session_id: "s".to_owned(),
        token: "k".to_owned(),
        tool: "t".to_owned(),
        event: "e".to_owned(),
        payload: b"{}".to_vec(),
        payload_omitted: true,
    });
    let fields = [
        0x0a, 0x01, b's', // field 1, bytes: session_id = "s"
        0x12, 0x01, b'k', // field 2, bytes: token = "k"
        0x1a, 0x01, b't', // field 3, bytes: tool = "t"
        0x22, 0x01, b'e', // field 4, bytes: event = "e"
        0x2a, 0x02, b'{', b'}', // field 5, bytes: payload
        0x30, 0x01, // field 6, varint: payload_omitted = true
    ];
    assert_eq!(request_bytes(hook), with_id(HOOK_EVENT, &fields));

    let plan = request::Body::HooksPlan(HooksPlanRequest {
        tool: "t".to_owned(),
    });
    assert_eq!(
        request_bytes(plan),
        with_id(HOOKS_PLAN, &[0x0a, 0x01, b't'])
    );
    let adapters = request::Body::HooksAdapters(HooksAdaptersRequest {});
    assert_eq!(request_bytes(adapters), with_id(HOOKS_ADAPTERS, &[]));
}

#[test]
fn hooks_responses_use_the_field_numbers_from_700() {
    let hook = response::Result::HookEvent(HookEventResponse {
        output: b"o".to_vec(),
    });
    assert_eq!(
        response_bytes(hook),
        with_id(HOOK_EVENT, &[0x0a, 0x01, b'o'])
    );

    let plan = response::Result::HooksPlan(HooksPlanResponse {
        extension_id: "x".to_owned(),
        plan: Some(HookInstallPlan::default()),
    });
    assert_eq!(
        response_bytes(plan),
        with_id(HOOKS_PLAN, &[0x0a, 0x01, b'x', 0x12, 0x00])
    );

    let adapters = response::Result::HooksAdapters(HooksAdaptersResponse {
        adapters: vec![HookAdapter {
            tool: "t".to_owned(),
            extension_id: "x".to_owned(),
        }],
    });
    let adapter = [0x0a, 0x06, 0x0a, 0x01, b't', 0x12, 0x01, b'x'];
    assert_eq!(response_bytes(adapters), with_id(HOOKS_ADAPTERS, &adapter));
}

#[test]
fn the_debug_output_of_a_hook_event_has_no_token() {
    let request = HookEventRequest {
        token: "secret-token".to_owned(),
        payload: b"secret payload".to_vec(),
        ..HookEventRequest::default()
    };

    let text = format!("{request:?}");

    assert!(text.contains("<redacted>"));
    assert!(!text.contains("secret"));
}

#[test]
fn an_extension_event_is_field_400_of_an_event() {
    let event = Event {
        seq: 1,
        at_ms: 2,
        body: Some(event::Body::ExtensionEvent(ExtensionEvent {
            extension_id: "x".to_owned(),
            name: "n".to_owned(),
            payload: b"p".to_vec(),
        })),
    };
    let expected = [
        0x08, 0x01, // field 1, varint: seq = 1
        0x10, 0x02, // field 2, varint: at_ms = 2
        0x82, 0x19, 0x09, // field 400, bytes
        0x0a, 0x01, b'x', // field 1, bytes: extension_id = "x"
        0x12, 0x01, b'n', // field 2, bytes: name = "n"
        0x1a, 0x01, b'p', // field 3, bytes: payload = "p"
    ];
    assert_eq!(event.encode_to_vec(), expected);
}

#[test]
fn the_details_of_an_agent_run_are_field_5() {
    let run = AgentRun {
        details: Some(AgentDetails {
            model: "m".to_owned(),
            provider_session_id: "p".to_owned(),
            resume_command: "r".to_owned(),
        }),
        ..AgentRun::default()
    };
    let expected = [
        0x2a, 0x09, // field 5, bytes: details
        0x0a, 0x01, b'm', // field 1, bytes: model = "m"
        0x12, 0x01, b'p', // field 2, bytes: provider_session_id = "p"
        0x1a, 0x01, b'r', // field 3, bytes: resume_command = "r"
    ];
    assert_eq!(run.encode_to_vec(), expected);
}

#[test]
fn the_payloads_of_the_methods_of_an_adapter_have_the_expected_bytes() {
    let event = HookEvent {
        session_id: "s".to_owned(),
        tool: "t".to_owned(),
        event: "e".to_owned(),
        payload: b"p".to_vec(),
        payload_omitted: true,
    };
    let expected = [
        0x0a, 0x01, b's', 0x12, 0x01, b't', 0x1a, 0x01, b'e', 0x22, 0x01, b'p', 0x28, 0x01,
    ];
    assert_eq!(event.encode_to_vec(), expected);
    let answer = HookAnswer {
        output: b"o".to_vec(),
    };
    assert_eq!(answer.encode_to_vec(), [0x0a, 0x01, b'o']);

    let plan = HookInstallPlan {
        settings_paths: vec![SettingsPath {
            platform: HookPlatform::Macos.into(),
            path: "~".to_owned(),
        }],
        entries: vec![SettingsEntry {
            pointer: "/".to_owned(),
            value_json: "1".to_owned(),
        }],
    };
    let expected = [
        0x0a, 0x05, 0x08, 0x02, 0x12, 0x01, b'~', // field 1: a path for MACOS
        0x12, 0x06, 0x0a, 0x01, b'/', 0x12, 0x01, b'1', // field 2: one entry
    ];
    assert_eq!(plan.encode_to_vec(), expected);
}

#[test]
fn the_payloads_of_the_host_services_for_an_adapter_have_the_expected_bytes() {
    let report = AgentReport {
        session_id: "s".to_owned(),
        state: AgentState::NeedsYou.into(),
        kind: NeedsYouKind::Question.into(),
        soft: true,
        message: "m".to_owned(),
        title: "t".to_owned(),
        tool: "c".to_owned(),
        details: Some(AgentDetails::default()),
    };
    let expected = [
        0x0a, 0x01, b's', // field 1: session_id
        0x10, 0x04, // field 2: state = NEEDS_YOU
        0x18, 0x02, // field 3: kind = QUESTION
        0x20, 0x01, // field 4: soft = true
        0x2a, 0x01, b'm', // field 5: message
        0x32, 0x01, b't', // field 6: title
        0x3a, 0x01, b'c', // field 7: tool
        0x42, 0x00, // field 8: details
    ];
    assert_eq!(report.encode_to_vec(), expected);

    let child = ChildWorkReport {
        session_id: "s".to_owned(),
        change: ChildWorkChange::End.into(),
        id: "i".to_owned(),
        parent_id: "p".to_owned(),
        state: AgentState::Done.into(),
        kind: NeedsYouKind::Auth.into(),
        soft: true,
        message: "m".to_owned(),
        title: "t".to_owned(),
        tool: "c".to_owned(),
    };
    let expected = [
        0x0a, 0x01, b's', // field 1: session_id
        0x10, 0x03, // field 2: change = END
        0x1a, 0x01, b'i', // field 3: id
        0x22, 0x01, b'p', // field 4: parent_id
        0x28, 0x03, // field 5: state = DONE
        0x30, 0x03, // field 6: kind = AUTH
        0x38, 0x01, // field 7: soft = true
        0x42, 0x01, b'm', // field 8: message
        0x4a, 0x01, b't', // field 9: title
        0x52, 0x01, b'c', // field 10: tool
    ];
    assert_eq!(child.encode_to_vec(), expected);
}
