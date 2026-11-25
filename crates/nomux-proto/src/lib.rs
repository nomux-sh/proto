//! The nomux wire protocol.
//!
//! This crate has three parts:
//!
//! - [`wire::v1`]: the control messages. `prost` generates these types from
//!   the schemas in `proto/nomux/wire/v1/`.
//! - [`frame`]: the fixed binary header of a terminal data frame.
//! - [`compatibility`]: the control protocol version and the version rule of
//!   the handshake.
//!
//! The generated code is in the repository. A build of this crate does not
//! start `buf` or `protoc`.
//!
//! The protocol is a draft. It can change until ADR-0002 is accepted.

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
    }
}
