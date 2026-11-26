//! Tests of the protobuf wire format of the hibernation of a Session: its
//! fields of a Session, its requests, its events and the memory limit.
//!
//! Field numbers, wire types and enum values are permanent. Each test has the
//! expected bytes or values as literals, thus a changed schema fails the test.

use nomux_proto::wire::v1::{
    Event, HibernationReason, HostInfoResponse, MemoryLimitSupport, MemoryMeasure, Request,
    Response, Session, SessionHibernateRequest, SessionHibernateResponse, SessionHibernated,
    SessionHibernation, SessionMemory, SessionMemoryLimitReached, SessionSetPinnedRequest,
    SessionSetPinnedResponse, SessionWakeFailed, SessionWoken, event, request, response,
};
use prost::Message;

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

fn event_bytes(body: event::Body) -> Vec<u8> {
    Event {
        seq: 3,
        at_ms: 4,
        body: Some(body),
    }
    .encode_to_vec()
}

#[test]
fn hibernation_reason_values_are_stable() {
    assert_eq!(HibernationReason::Unspecified as i32, 0);
    assert_eq!(HibernationReason::Count as i32, 1);
    assert_eq!(HibernationReason::SessionMemory as i32, 2);
    assert_eq!(HibernationReason::TotalMemory as i32, 3);
    assert_eq!(HibernationReason::MemoryPressure as i32, 4);
    assert_eq!(HibernationReason::Request as i32, 5);
}

#[test]
fn the_hibernation_of_a_session_is_field_14_and_the_pin_is_field_15() {
    let session = Session {
        id: "s".to_owned(),
        hibernation: Some(SessionHibernation {
            hibernated_at: 5,
            reason: HibernationReason::Count.into(),
            freed_bytes: 300,
            measure: MemoryMeasure::Proportional.into(),
            resume_command: "a".to_owned(),
            woken_at: 6,
            wake_error: "e".to_owned(),
        }),
        pinned: true,
        ..Session::default()
    };
    assert_eq!(
        session.encode_to_vec(),
        [
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x72, 0x11, // field 14, bytes: hibernation
            0x08, 0x05, // SessionHibernation field 1, varint: hibernated_at = 5
            0x10, 0x01, // field 2, varint: reason = COUNT
            0x18, 0xac, 0x02, // field 3, varint: freed_bytes = 300
            0x20, 0x01, // field 4, varint: measure = PROPORTIONAL
            0x2a, 0x01, b'a', // field 5, bytes: resume_command = "a"
            0x30, 0x06, // field 6, varint: woken_at = 6
            0x3a, 0x01, b'e', // field 7, bytes: wake_error = "e"
            0x78, 0x01, // field 15, varint: pinned = true
        ],
    );
}

#[test]
fn the_hibernation_requests_are_fields_311_and_312() {
    assert_eq!(
        request_bytes(request::Body::SessionHibernate(SessionHibernateRequest {
            id: "s".to_owned(),
        })),
        [
            0x08, 0x01, // id = 1
            0xba, 0x13, 0x03, // field 311, bytes: session_hibernate
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionHibernate(
            SessionHibernateResponse { session: None }
        )),
        [0x08, 0x01, 0xba, 0x13, 0x00],
    );
    assert_eq!(
        request_bytes(request::Body::SessionSetPinned(SessionSetPinnedRequest {
            id: "s".to_owned(),
            pinned: true,
        })),
        [
            0x08, 0x01, // id = 1
            0xc2, 0x13, 0x05, // field 312, bytes: session_set_pinned
            0x0a, 0x01, b's', // field 1, bytes: id = "s"
            0x10, 0x01, // field 2, varint: pinned = true
        ],
    );
    assert_eq!(
        response_bytes(response::Result::SessionSetPinned(
            SessionSetPinnedResponse {}
        )),
        [0x08, 0x01, 0xc2, 0x13, 0x00],
    );
}

#[test]
fn the_memory_limit_of_a_session_is_field_6() {
    let memory = SessionMemory {
        limit_bytes: 300,
        ..SessionMemory::default()
    };
    assert_eq!(memory.encode_to_vec(), [0x30, 0xac, 0x02]);
}

#[test]
fn the_memory_limit_support_of_a_host_is_field_4_of_host_info() {
    let info = HostInfoResponse {
        memory_limit: Some(MemoryLimitSupport {
            applied: true,
            detail: "d".to_owned(),
        }),
        ..HostInfoResponse::default()
    };
    assert_eq!(
        info.encode_to_vec(),
        [
            0x22, 0x05, // field 4, bytes: memory_limit
            0x08, 0x01, // field 1, varint: applied = true
            0x12, 0x01, b'd', // field 2, bytes: detail = "d"
        ],
    );
}

#[test]
fn the_session_events_are_in_the_block_of_the_session_service() {
    let header = [
        0x08, 0x03, // field 1, varint: seq = 3
        0x10, 0x04, // field 2, varint: at_ms = 4
    ];
    let expected = |tag: [u8; 2], message: &[u8]| {
        let mut bytes = header.to_vec();
        bytes.extend(tag);
        bytes.push(u8::try_from(message.len()).unwrap());
        bytes.extend(message);
        bytes
    };

    let hibernated = SessionHibernated {
        session_id: "s".to_owned(),
        reason: HibernationReason::MemoryPressure.into(),
        freed_bytes: 1,
        measure: MemoryMeasure::Resident.into(),
        live_agents: 2,
        max_live_agents: 3,
        memory_bytes: 4,
        memory_limit_bytes: 5,
        pressure_stall_ms: 6,
        pressure_window_ms: 7,
    };
    assert_eq!(
        event_bytes(event::Body::SessionHibernated(hibernated)),
        expected(
            [0xe2, 0x12], // field 300
            &[
                0x0a, 0x01, b's', // field 1, bytes: session_id = "s"
                0x10, 0x04, // field 2, varint: reason = MEMORY_PRESSURE
                0x18, 0x01, // field 3, varint: freed_bytes = 1
                0x20, 0x02, // field 4, varint: measure = RESIDENT
                0x28, 0x02, // field 5, varint: live_agents = 2
                0x30, 0x03, // field 6, varint: max_live_agents = 3
                0x38, 0x04, // field 7, varint: memory_bytes = 4
                0x40, 0x05, // field 8, varint: memory_limit_bytes = 5
                0x48, 0x06, // field 9, varint: pressure_stall_ms = 6
                0x50, 0x07, // field 10, varint: pressure_window_ms = 7
            ],
        ),
    );
    assert_eq!(
        event_bytes(event::Body::SessionWoken(SessionWoken {
            session_id: "s".to_owned(),
            resume_command: "a".to_owned(),
        })),
        expected([0xea, 0x12], &[0x0a, 0x01, b's', 0x12, 0x01, b'a']), // field 301
    );
    assert_eq!(
        event_bytes(event::Body::SessionWakeFailed(SessionWakeFailed {
            session_id: "s".to_owned(),
            error: "e".to_owned(),
            resume_command: "a".to_owned(),
        })),
        expected(
            [0xf2, 0x12], // field 302
            &[0x0a, 0x01, b's', 0x12, 0x01, b'e', 0x1a, 0x01, b'a'],
        ),
    );
    assert_eq!(
        event_bytes(event::Body::SessionMemoryLimitReached(
            SessionMemoryLimitReached {
                session_id: "s".to_owned(),
                limit_bytes: 300,
            }
        )),
        expected([0xfa, 0x12], &[0x0a, 0x01, b's', 0x10, 0xac, 0x02]), // field 303
    );
}
