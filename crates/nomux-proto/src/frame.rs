//! The fixed binary header of a terminal data frame.
//!
//! Terminal data does not use protobuf. Each frame is a 24-byte header and
//! then `length` bytes of payload. The exact layout is specified in
//! `spec/frame.md`.
//!
//! STATUS: DRAFT. The layout can change until ADR-0002 is accepted.
//!
//! All integers are little-endian.
//!
//! | Offset | Size | Field     |
//! |-------:|-----:|-----------|
//! |      0 |    1 | `version` |
//! |      1 |    1 | `kind`    |
//! |      2 |    2 | `flags`   |
//! |      4 |    4 | `session` |
//! |      8 |    4 | `epoch`   |
//! |     12 |    4 | `length`  |
//! |     16 |    8 | `seq`     |
//!
//! This module defines only the header. The payload layout of each frame
//! kind is not defined yet.

use core::fmt;

/// The length of the frame header in bytes.
pub const HEADER_LEN: usize = 24;

/// The header layout version that this module implements.
pub const FRAME_VERSION: u8 = 1;

/// The largest permitted value of `length` (1 MiB).
pub const MAX_PAYLOAD_LEN: u32 = 1 << 20;

/// The kind of a terminal data frame.
///
/// The discriminant is the value of the `kind` byte. The value 0 is not a
/// valid kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum FrameKind {
    /// Terminal output bytes of a session.
    Output = 1,
    /// A full copy of the screen.
    ScreenSnapshot = 2,
    /// The rows that changed on the screen.
    ScreenDiff = 3,
    /// The receiver confirms the frames up to a sequence number.
    Ack = 4,
    /// The receiver gives flow-control credit to the sender.
    Credit = 5,
    /// Input bytes for a session. Used on the input channel.
    Input = 6,
}

impl FrameKind {
    /// All frame kinds, in the order of their byte values.
    pub const ALL: [Self; 6] = [
        Self::Output,
        Self::ScreenSnapshot,
        Self::ScreenDiff,
        Self::Ack,
        Self::Credit,
        Self::Input,
    ];

    /// Returns the frame kind for a `kind` byte, or `None` if the value is
    /// not registered.
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Output),
            2 => Some(Self::ScreenSnapshot),
            3 => Some(Self::ScreenDiff),
            4 => Some(Self::Ack),
            5 => Some(Self::Credit),
            6 => Some(Self::Input),
            _ => None,
        }
    }

    /// Returns the value of the `kind` byte.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A frame header that is not valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// The input has fewer bytes than [`HEADER_LEN`].
    Truncated {
        /// The number of bytes in the input.
        available: usize,
    },
    /// The `version` byte is not [`FRAME_VERSION`]. The other fields were
    /// not read.
    UnsupportedVersion(u8),
    /// The `flags` field is not 0.
    ReservedFlags(u16),
    /// The `length` field is larger than [`MAX_PAYLOAD_LEN`].
    PayloadTooLarge {
        /// The value of the `length` field.
        length: u32,
    },
    /// The `kind` byte is not registered. The other fields are valid, thus
    /// `length` is the correct number of payload bytes.
    UnknownKind {
        /// The value of the `kind` byte.
        kind: u8,
        /// The value of the `length` field.
        length: u32,
    },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { available } => write!(
                f,
                "frame header is truncated: {available} of {HEADER_LEN} bytes"
            ),
            Self::UnsupportedVersion(version) => {
                write!(f, "frame header version {version} is not supported")
            }
            Self::ReservedFlags(flags) => {
                write!(f, "reserved frame flags are set: {flags:#06x}")
            }
            Self::PayloadTooLarge { length } => write!(
                f,
                "frame payload length {length} is larger than {MAX_PAYLOAD_LEN}"
            ),
            Self::UnknownKind { kind, .. } => write!(f, "frame kind {kind} is not known"),
        }
    }
}

impl core::error::Error for FrameError {}

/// The header of a terminal data frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    /// The kind of the frame.
    pub kind: FrameKind,
    /// The session handle. The handle identifies a session on one connection.
    pub session: u32,
    /// The epoch of the session.
    pub epoch: u32,
    /// The sequence number in the epoch.
    pub seq: u64,
    /// The number of payload bytes after the header.
    pub length: u32,
}

impl FrameHeader {
    /// Encodes the header.
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::PayloadTooLarge`] if `length` is larger than
    /// [`MAX_PAYLOAD_LEN`].
    pub fn encode(&self) -> Result<[u8; HEADER_LEN], FrameError> {
        if self.length > MAX_PAYLOAD_LEN {
            return Err(FrameError::PayloadTooLarge {
                length: self.length,
            });
        }
        let mut bytes = [0u8; HEADER_LEN];
        bytes[0] = FRAME_VERSION;
        bytes[1] = self.kind.as_u8();
        // Bytes 2 and 3 are the reserved `flags` field and stay 0.
        bytes[4..8].copy_from_slice(&self.session.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.epoch.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.length.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.seq.to_le_bytes());
        Ok(bytes)
    }

    /// Decodes a header.
    ///
    /// The checks are done in this order: `version`, `flags`, `length`,
    /// `kind`.
    ///
    /// # Errors
    ///
    /// Returns a [`FrameError`] for the first check that fails.
    pub fn decode(bytes: &[u8; HEADER_LEN]) -> Result<Self, FrameError> {
        let [
            version,
            kind,
            f0,
            f1,
            s0,
            s1,
            s2,
            s3,
            e0,
            e1,
            e2,
            e3,
            l0,
            l1,
            l2,
            l3,
            q0,
            q1,
            q2,
            q3,
            q4,
            q5,
            q6,
            q7,
        ] = *bytes;

        if version != FRAME_VERSION {
            return Err(FrameError::UnsupportedVersion(version));
        }
        let flags = u16::from_le_bytes([f0, f1]);
        if flags != 0 {
            return Err(FrameError::ReservedFlags(flags));
        }
        let length = u32::from_le_bytes([l0, l1, l2, l3]);
        if length > MAX_PAYLOAD_LEN {
            return Err(FrameError::PayloadTooLarge { length });
        }
        let Some(kind) = FrameKind::from_u8(kind) else {
            return Err(FrameError::UnknownKind { kind, length });
        };
        Ok(Self {
            kind,
            session: u32::from_le_bytes([s0, s1, s2, s3]),
            epoch: u32::from_le_bytes([e0, e1, e2, e3]),
            seq: u64::from_le_bytes([q0, q1, q2, q3, q4, q5, q6, q7]),
            length,
        })
    }

    /// Decodes a header from the start of `bytes`.
    ///
    /// Returns the header and the bytes after the header.
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::Truncated`] if `bytes` has fewer bytes than
    /// [`HEADER_LEN`]. Returns the error of [`FrameHeader::decode`] if the
    /// header is not valid.
    pub fn decode_prefix(bytes: &[u8]) -> Result<(Self, &[u8]), FrameError> {
        let Some((header, rest)) = bytes.split_first_chunk::<HEADER_LEN>() else {
            return Err(FrameError::Truncated {
                available: bytes.len(),
            });
        };
        Ok((Self::decode(header)?, rest))
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// A valid encoded header. Each field has a value with different bytes,
    /// thus a wrong offset or a wrong byte order fails the test.
    const SAMPLE_BYTES: [u8; HEADER_LEN] = [
        0x01, // version
        0x01, // kind: Output
        0x00, 0x00, // flags
        0x04, 0x03, 0x02, 0x01, // session
        0x08, 0x07, 0x06, 0x05, // epoch
        0x0c, 0x0b, 0x0a, 0x00, // length
        0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11, // seq
    ];

    const SAMPLE_HEADER: FrameHeader = FrameHeader {
        kind: FrameKind::Output,
        session: 0x0102_0304,
        epoch: 0x0506_0708,
        seq: 0x1112_1314_1516_1718,
        length: 0x000a_0b0c,
    };

    #[test]
    fn encode_matches_the_specified_layout() {
        assert_eq!(SAMPLE_HEADER.encode(), Ok(SAMPLE_BYTES));
    }

    #[test]
    fn decode_matches_the_specified_layout() {
        assert_eq!(FrameHeader::decode(&SAMPLE_BYTES), Ok(SAMPLE_HEADER));
    }

    #[test]
    fn kind_bytes_are_stable() {
        let expected = [
            (FrameKind::Output, 1),
            (FrameKind::ScreenSnapshot, 2),
            (FrameKind::ScreenDiff, 3),
            (FrameKind::Ack, 4),
            (FrameKind::Credit, 5),
            (FrameKind::Input, 6),
        ];
        assert_eq!(expected.map(|(kind, _)| kind), FrameKind::ALL);
        for (kind, byte) in expected {
            assert_eq!(kind.as_u8(), byte);
            assert_eq!(FrameKind::from_u8(byte), Some(kind));
        }
        for byte in [0, 7, 0x80, 0xff] {
            assert_eq!(FrameKind::from_u8(byte), None);
        }
    }

    #[test]
    fn decode_rejects_invalid_headers() {
        fn with(offset: usize, replacement: &[u8]) -> [u8; HEADER_LEN] {
            let mut bytes = SAMPLE_BYTES;
            bytes[offset..offset + replacement.len()].copy_from_slice(replacement);
            bytes
        }
        let too_large = MAX_PAYLOAD_LEN + 1;

        let cases = [
            (with(0, &[0]), FrameError::UnsupportedVersion(0)),
            (with(0, &[2]), FrameError::UnsupportedVersion(2)),
            (with(2, &[1, 0]), FrameError::ReservedFlags(0x0001)),
            (with(2, &[0, 0x80]), FrameError::ReservedFlags(0x8000)),
            (
                with(12, &too_large.to_le_bytes()),
                FrameError::PayloadTooLarge { length: too_large },
            ),
            (
                with(1, &[0]),
                FrameError::UnknownKind {
                    kind: 0,
                    length: SAMPLE_HEADER.length,
                },
            ),
            (
                with(1, &[7]),
                FrameError::UnknownKind {
                    kind: 7,
                    length: SAMPLE_HEADER.length,
                },
            ),
        ];
        for (bytes, expected) in cases {
            assert_eq!(FrameHeader::decode(&bytes), Err(expected), "{bytes:?}");
        }
    }

    #[test]
    fn checks_run_in_the_specified_order() {
        // All four checks fail. The error is for `version`.
        let mut bytes = [0xff; HEADER_LEN];
        assert_eq!(
            FrameHeader::decode(&bytes),
            Err(FrameError::UnsupportedVersion(0xff))
        );
        bytes[0] = FRAME_VERSION;
        assert_eq!(
            FrameHeader::decode(&bytes),
            Err(FrameError::ReservedFlags(0xffff))
        );
        bytes[2..4].fill(0);
        assert_eq!(
            FrameHeader::decode(&bytes),
            Err(FrameError::PayloadTooLarge { length: u32::MAX })
        );
        bytes[12..16].fill(0);
        assert_eq!(
            FrameHeader::decode(&bytes),
            Err(FrameError::UnknownKind {
                kind: 0xff,
                length: 0
            })
        );
    }

    #[test]
    fn length_limit_is_inclusive() {
        let mut header = SAMPLE_HEADER;
        header.length = MAX_PAYLOAD_LEN;
        let bytes = header.encode().expect("the limit is a valid length");
        assert_eq!(FrameHeader::decode(&bytes), Ok(header));

        header.length = MAX_PAYLOAD_LEN + 1;
        assert_eq!(
            header.encode(),
            Err(FrameError::PayloadTooLarge {
                length: MAX_PAYLOAD_LEN + 1
            })
        );
    }

    #[test]
    fn decode_prefix_returns_the_remaining_bytes() {
        let mut bytes = SAMPLE_BYTES.to_vec();
        bytes.extend_from_slice(b"payload");
        assert_eq!(
            FrameHeader::decode_prefix(&bytes),
            Ok((SAMPLE_HEADER, b"payload".as_slice()))
        );
    }

    #[test]
    fn decode_prefix_rejects_short_input() {
        for available in 0..HEADER_LEN {
            assert_eq!(
                FrameHeader::decode_prefix(&SAMPLE_BYTES[..available]),
                Err(FrameError::Truncated { available })
            );
        }
    }

    fn arbitrary_header() -> impl Strategy<Value = FrameHeader> {
        (
            proptest::sample::select(FrameKind::ALL.to_vec()),
            any::<u32>(),
            any::<u32>(),
            any::<u64>(),
            0..=MAX_PAYLOAD_LEN,
        )
            .prop_map(|(kind, session, epoch, seq, length)| FrameHeader {
                kind,
                session,
                epoch,
                seq,
                length,
            })
    }

    proptest! {
        #[test]
        fn encode_then_decode_returns_the_header(header in arbitrary_header()) {
            let bytes = header.encode().expect("the header is valid");
            prop_assert_eq!(FrameHeader::decode(&bytes), Ok(header));
        }

        #[test]
        fn decode_then_encode_returns_the_bytes(bytes in any::<[u8; HEADER_LEN]>()) {
            // Decode must not panic for any input. If decode accepts the
            // bytes, the encoding is canonical.
            if let Ok(header) = FrameHeader::decode(&bytes) {
                prop_assert_eq!(header.encode(), Ok(bytes));
            }
        }

        #[test]
        fn decode_accepts_only_valid_fixed_fields(
            header in arbitrary_header(),
            version in any::<u8>(),
            kind in any::<u8>(),
            flags in any::<u16>(),
        ) {
            let mut bytes = header.encode().expect("the header is valid");
            bytes[0] = version;
            bytes[1] = kind;
            bytes[2..4].copy_from_slice(&flags.to_le_bytes());
            let valid = version == FRAME_VERSION
                && flags == 0
                && FrameKind::from_u8(kind).is_some();
            prop_assert_eq!(FrameHeader::decode(&bytes).is_ok(), valid);
        }
    }
}
