//! Tests of the protobuf wire format of the Agent service and of the Events
//! service.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    AgentExplainRequest, AgentExplainResponse, AgentRun, AgentRunChanged, AgentRunRemoved,
    AgentSetStatusRequest, AgentSetStatusResponse, AgentState, AgentStatus, AgentTreeRequest,
    AgentTreeResponse, ChildWork, Event, EventType, EventsSnapshot, EventsSubscribeRequest,
    EventsSubscribeResponse, NeedsYouKind, Request, Response, SourceReport, StatusProgress,
    StatusSource, StatusText, event, request, response,
};
use prost::Message;

/// The first byte of the tag of a field from 500 to 511 with a length. The
/// second byte is 0x1f.
const AGENT_TREE: u8 = 0xa2;
const AGENT_EXPLAIN: u8 = 0xaa;
const AGENT_SET_STATUS: u8 = 0xb2;
/// The two bytes of the tag of field 600 with a length.
const EVENTS_FIELD: [u8; 2] = [0xc2, 0x25];

fn status() -> AgentStatus {
    AgentStatus {
        state: AgentState::NeedsYou.into(),
        kind: NeedsYouKind::Permission.into(),
        source: StatusSource::ProgramStatus.into(),
        since_ms: 7,
        seen: true,
        message: Some(StatusText {
            text: "m".to_owned(),
            source: StatusSource::SelfReport.into(),
        }),
        title: Some(StatusText {
            text: "t".to_owned(),
            source: StatusSource::ProgramStatus.into(),
        }),
        progress: Some(StatusProgress {
            percent: 9,
            source: StatusSource::ProgramStatus.into(),
        }),
        tool: Some(StatusText {
            text: "c".to_owned(),
            source: StatusSource::ProgramStatus.into(),
        }),
    }
}

const STATUS_BYTES: [u8; 35] = [
    0x08, 0x04, // field 1, varint: state = NEEDS_YOU
    0x10, 0x01, // field 2, varint: kind = PERMISSION
    0x18, 0x14, // field 3, varint: source = PROGRAM_STATUS
    0x20, 0x07, // field 4, varint: since_ms = 7
    0x28, 0x01, // field 5, varint: seen = true
    0x32, 0x05, 0x0a, 0x01, b'm', 0x10, 0x1e, // field 6: message, from SELF_REPORT
    0x3a, 0x05, 0x0a, 0x01, b't', 0x10, 0x14, // field 7: title
    0x42, 0x04, 0x08, 0x09, 0x10, 0x14, // field 8: progress = 9
    0x4a, 0x05, 0x0a, 0x01, b'c', // field 9: tool, the source follows
];

fn idle() -> AgentStatus {
    AgentStatus {
        state: AgentState::Idle.into(),
        source: StatusSource::None.into(),
        ..AgentStatus::default()
    }
}

/// `idle()`: state = IDLE, source = NONE.
const IDLE_BYTES: [u8; 4] = [0x08, 0x01, 0x18, 0x01];

fn run() -> AgentRun {
    AgentRun {
        session_id: "s".to_owned(),
        status: Some(idle()),
        aggregate: Some(idle()),
        children: vec![ChildWork {
            name: "n".to_owned(),
            status: Some(idle()),
            aggregate: Some(idle()),
            children: vec![ChildWork {
                name: "o".to_owned(),
                ..ChildWork::default()
            }],
        }],
        details: None,
    }
}

fn run_bytes() -> Vec<u8> {
    let mut bytes = vec![0x0a, 0x01, b's']; // field 1, bytes: session_id = "s"
    bytes.extend([0x12, 0x04]); // field 2, bytes: status
    bytes.extend(IDLE_BYTES);
    bytes.extend([0x1a, 0x04]); // field 3, bytes: aggregate
    bytes.extend(IDLE_BYTES);
    bytes.extend([0x22, 0x14]); // field 4, bytes: one Child work
    bytes.extend([0x0a, 0x01, b'n']); // field 1, bytes: name = "n"
    bytes.extend([0x12, 0x04]); // field 2, bytes: status
    bytes.extend(IDLE_BYTES);
    bytes.extend([0x1a, 0x04]); // field 3, bytes: aggregate
    bytes.extend(IDLE_BYTES);
    bytes.extend([0x22, 0x03, 0x0a, 0x01, b'o']); // field 4: a Child work "o"
    bytes
}

fn length_prefixed(tag: &[u8], message: &[u8]) -> Vec<u8> {
    let length = u8::try_from(message.len()).expect("a short message");
    let mut bytes = tag.to_vec();
    bytes.push(length);
    bytes.extend_from_slice(message);
    bytes
}

/// The bytes of a request or a response with the `id` 1 and one field.
fn with_id(tag: &[u8], message: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x08, 0x01];
    bytes.extend(length_prefixed(tag, message));
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
fn agent_state_and_kind_values_are_stable() {
    assert_eq!(AgentState::Unspecified as i32, 0);
    assert_eq!(AgentState::Idle as i32, 1);
    assert_eq!(AgentState::Working as i32, 2);
    assert_eq!(AgentState::Done as i32, 3);
    assert_eq!(AgentState::NeedsYou as i32, 4);
    assert_eq!(AgentState::Error as i32, 5);

    assert_eq!(NeedsYouKind::Unspecified as i32, 0);
    assert_eq!(NeedsYouKind::Permission as i32, 1);
    assert_eq!(NeedsYouKind::Question as i32, 2);
    assert_eq!(NeedsYouKind::Auth as i32, 3);
}

#[test]
fn status_source_values_are_stable_and_leave_room_between_the_sources() {
    assert_eq!(StatusSource::Unspecified as i32, 0);
    assert_eq!(StatusSource::None as i32, 1);
    assert_eq!(StatusSource::Adapter as i32, 10);
    assert_eq!(StatusSource::ProgramStatus as i32, 20);
    assert_eq!(StatusSource::SelfReport as i32, 30);
    assert_eq!(StatusSource::SessionRestore as i32, 40);
    assert_eq!(StatusSource::PromptMarks as i32, 50);
    assert_eq!(StatusSource::ForegroundProgram as i32, 60);
}

#[test]
fn event_type_values_are_stable() {
    assert_eq!(EventType::Unspecified as i32, 0);
    assert_eq!(EventType::AgentRunChanged as i32, 1);
    assert_eq!(EventType::AgentRunRemoved as i32, 2);
    assert_eq!(EventType::ExtensionEvent as i32, 3);
}

#[test]
fn agent_status_has_the_expected_bytes() {
    let mut expected = STATUS_BYTES.to_vec();
    expected.extend([0x10, 0x14]); // the source of the tool
    assert_eq!(status().encode_to_vec(), expected);
    assert_eq!(AgentStatus::decode(expected.as_slice()), Ok(status()));
}

#[test]
fn agent_run_has_the_expected_bytes() {
    assert_eq!(run().encode_to_vec(), run_bytes());
    assert_eq!(AgentRun::decode(run_bytes().as_slice()), Ok(run()));
}

#[test]
fn agent_requests_use_the_field_numbers_from_500() {
    let tree = request::Body::AgentTree(AgentTreeRequest {
        include_children: true,
    });
    assert_eq!(
        request_bytes(tree),
        with_id(&[AGENT_TREE, 0x1f], &[0x08, 0x01])
    );

    let explain = request::Body::AgentExplain(AgentExplainRequest {
        session_id: "s".to_owned(),
    });
    assert_eq!(
        request_bytes(explain),
        with_id(&[AGENT_EXPLAIN, 0x1f], &[0x0a, 0x01, b's'])
    );

    let set_status = request::Body::AgentSetStatus(AgentSetStatusRequest {
        session_id: "s".to_owned(),
        token: "k".to_owned(),
        state: AgentState::Working.into(),
        message: "m".to_owned(),
    });
    let fields = [
        0x0a, 0x01, b's', // field 1, bytes: session_id = "s"
        0x12, 0x01, b'k', // field 2, bytes: token = "k"
        0x18, 0x02, // field 3, varint: state = WORKING
        0x22, 0x01, b'm', // field 4, bytes: message = "m"
    ];
    assert_eq!(
        request_bytes(set_status),
        with_id(&[AGENT_SET_STATUS, 0x1f], &fields)
    );
}

#[test]
fn agent_responses_use_the_field_numbers_from_500() {
    let tree = response::Result::AgentTree(Box::new(AgentTreeResponse {
        runs: vec![AgentRun {
            session_id: "s".to_owned(),
            ..AgentRun::default()
        }],
        attention_session_id: "s".to_owned(),
        aggregate: Some(idle()),
    }));
    let fields = [
        0x0a, 0x03, 0x0a, 0x01, b's', // field 1, bytes: one run
        0x12, 0x01, b's', // field 2, bytes: attention_session_id = "s"
        0x1a, 0x04, 0x08, 0x01, 0x18, 0x01, // field 3, bytes: aggregate
    ];
    assert_eq!(response_bytes(tree), with_id(&[AGENT_TREE, 0x1f], &fields));

    let explain = response::Result::AgentExplain(Box::new(AgentExplainResponse {
        run: Some(AgentRun {
            session_id: "s".to_owned(),
            ..AgentRun::default()
        }),
        sources: vec![SourceReport {
            source: StatusSource::PromptMarks.into(),
            reported: true,
            state: AgentState::Working.into(),
            kind: NeedsYouKind::Question.into(),
            message: "m".to_owned(),
            note: "n".to_owned(),
        }],
    }));
    let fields = [
        0x0a, 0x03, 0x0a, 0x01, b's', // field 1, bytes: run
        0x12, 0x0e, // field 2, bytes: one source
        0x08, 0x32, // field 1, varint: source = PROMPT_MARKS
        0x10, 0x01, // field 2, varint: reported = true
        0x18, 0x02, // field 3, varint: state = WORKING
        0x20, 0x02, // field 4, varint: kind = QUESTION
        0x2a, 0x01, b'm', // field 5, bytes: message = "m"
        0x32, 0x01, b'n', // field 6, bytes: note = "n"
    ];
    assert_eq!(
        response_bytes(explain),
        with_id(&[AGENT_EXPLAIN, 0x1f], &fields)
    );

    let set_status = response::Result::AgentSetStatus(AgentSetStatusResponse {});
    assert_eq!(
        response_bytes(set_status),
        with_id(&[AGENT_SET_STATUS, 0x1f], &[])
    );
}

#[test]
fn events_subscribe_uses_the_field_number_600() {
    let subscribe = request::Body::EventsSubscribe(EventsSubscribeRequest {
        from_seq: 5,
        types: vec![
            EventType::AgentRunChanged.into(),
            EventType::AgentRunRemoved.into(),
        ],
    });
    let fields = [
        0x08, 0x05, // field 1, varint: from_seq = 5
        0x12, 0x02, 0x01, 0x02, // field 2, packed: the two types
    ];
    assert_eq!(request_bytes(subscribe), with_id(&EVENTS_FIELD, &fields));

    let subscribed = response::Result::EventsSubscribe(EventsSubscribeResponse { last_seq: 9 });
    assert_eq!(
        response_bytes(subscribed),
        with_id(&EVENTS_FIELD, &[0x08, 0x09])
    );
}

#[test]
fn an_event_has_its_body_in_the_block_of_its_service() {
    let header = [
        0x08, 0x03, // field 1, varint: seq = 3
        0x10, 0x04, // field 2, varint: at_ms = 4
    ];
    let event = |body| Event {
        seq: 3,
        at_ms: 4,
        body: Some(body),
    };
    let expected = |tag: &[u8], message: &[u8]| {
        let mut bytes = header.to_vec();
        bytes.extend(length_prefixed(tag, message));
        bytes
    };

    let changed = event(event::Body::AgentRunChanged(Box::new(AgentRunChanged {
        run: Some(AgentRun {
            session_id: "s".to_owned(),
            ..AgentRun::default()
        }),
    })));
    assert_eq!(
        changed.encode_to_vec(),
        expected(&[0xa2, 0x1f], &[0x0a, 0x03, 0x0a, 0x01, b's'])
    );

    let removed = event(event::Body::AgentRunRemoved(AgentRunRemoved {
        session_id: "s".to_owned(),
    }));
    assert_eq!(
        removed.encode_to_vec(),
        expected(&[0xaa, 0x1f], &[0x0a, 0x01, b's'])
    );

    let snapshot = event(event::Body::Snapshot(EventsSnapshot {}));
    assert_eq!(snapshot.encode_to_vec(), expected(&EVENTS_FIELD, &[]));
}

#[test]
fn an_event_without_a_body_that_the_reader_knows_keeps_its_sequence_number() {
    // Field 700 is the first number of the next block: a later service.
    let bytes = [0x08, 0x03, 0xe2, 0x2b, 0x00];
    let event = Event::decode(bytes.as_slice()).unwrap();
    assert_eq!((event.seq, event.body), (3, None));
}

#[test]
fn set_status_debug_output_does_not_contain_the_token() {
    let request = Request {
        id: 1,
        body: Some(request::Body::AgentSetStatus(AgentSetStatusRequest {
            session_id: "s".to_owned(),
            token: "example-token".to_owned(),
            state: AgentState::Done.into(),
            message: "m".to_owned(),
        })),
    };
    let debug = format!("{request:?}");
    assert!(!debug.contains("example-token"), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
}
