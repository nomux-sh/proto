//! The nomux wire protocol.
//!
//! This crate has four parts:
//!
//! - [`wire::v1`]: the control messages: the envelope, the handshake, the
//!   Host service, the Session service, the Ext service, the Agent service,
//!   the Events service and the Hooks service, and the payloads of the
//!   adapter extensions. `prost` generates these types from the schemas in
//!   `proto/nomux/wire/v1/`.
//! - [`frame`]: the fixed binary header of a terminal data frame and the
//!   payload of the `End` frame.
//! - [`compatibility`]: the control protocol version and the version rule of
//!   the handshake.
//! - [`capabilities`]: the names of the capability flags.
//!
//! The generated code is a part of the crate. A build of this crate does not
//! start `buf` or `protoc`.
//!
//! Versions earlier than 1.0.0 are pre-releases: interfaces can change
//! between minor versions. The frame header layout is a draft.

pub mod capabilities;
pub mod compatibility;
pub mod frame;

/// Compiles and runs the example in the crate README as a documentation test.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

/// The control messages of the nomux wire protocol.
pub mod wire {
    /// Package `nomux.wire.v1`.
    ///
    /// `buf generate` writes the included file. Do not edit the file. The
    /// file is included with `include!`, thus `rustfmt` does not format it.
    pub mod v1 {
        include!("generated/nomux.wire.v1.rs");

        /// The largest number of bytes of one encoded [`Envelope`] on the
        /// control channel. A peer that reads a larger length closes the
        /// connection. See `spec/control.md`.
        pub const MAX_ENVELOPE_LENGTH: usize = 1024 * 1024;

        /// `Hello` contains the device token, which is a secret. This
        /// implementation prints all fields but replaces the token.
        impl core::fmt::Debug for Hello {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let token = if self.token.is_empty() {
                    "<empty>"
                } else {
                    "<redacted>"
                };
                f.debug_struct("Hello")
                    .field("version", &self.version)
                    .field("oldest", &self.oldest)
                    .field("capabilities", &self.capabilities)
                    .field("client_kind", &self.client_kind)
                    .field("token", &token)
                    .finish()
            }
        }

        /// `AgentSetStatusRequest` contains the token of a Session, which
        /// is a secret. This implementation prints all fields but replaces
        /// the token.
        impl core::fmt::Debug for AgentSetStatusRequest {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let token = if self.token.is_empty() {
                    "<empty>"
                } else {
                    "<redacted>"
                };
                f.debug_struct("AgentSetStatusRequest")
                    .field("session_id", &self.session_id)
                    .field("token", &token)
                    .field("state", &self.state)
                    .field("message", &self.message)
                    .finish()
            }
        }

        /// `HookEventRequest` contains the token of a Session, which is a
        /// secret. This implementation prints all fields but replaces the
        /// token, and prints the length of the payload.
        impl core::fmt::Debug for HookEventRequest {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let token = if self.token.is_empty() {
                    "<empty>"
                } else {
                    "<redacted>"
                };
                f.debug_struct("HookEventRequest")
                    .field("session_id", &self.session_id)
                    .field("token", &token)
                    .field("tool", &self.tool)
                    .field("event", &self.event)
                    .field("payload_length", &self.payload.len())
                    .field("payload_omitted", &self.payload_omitted)
                    .finish()
            }
        }
    }
}
