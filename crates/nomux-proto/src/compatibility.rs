//! The control protocol version and the version rule of the handshake.
//!
//! The rule is specified in `spec/compatibility.md`.

use core::fmt;

/// The newest control protocol version that this crate implements.
///
/// Until version 1.0.0 of the product, builds with the same value can be
/// incompatible. See `spec/compatibility.md`.
pub const PROTOCOL_VERSION: u32 = 1;

/// The oldest control protocol version that this crate can use.
pub const OLDEST_SUPPORTED_VERSION: u32 = 1;

/// The exit code of the `nomux` command-line client when the versions are
/// not compatible.
pub const EXIT_CODE_VERSION_INCOMPATIBLE: u8 = 5;

/// The range of control protocol versions that one peer can use.
///
/// The values come from the `version` and `oldest` fields of `Hello` or
/// `Welcome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VersionRange {
    /// The newest version that the peer can use.
    pub version: u32,
    /// The oldest version that the peer can use.
    pub oldest: u32,
}

impl VersionRange {
    /// The range that this crate implements.
    pub const CURRENT: Self = Self {
        version: PROTOCOL_VERSION,
        oldest: OLDEST_SUPPORTED_VERSION,
    };

    /// Returns `true` if `oldest` is not 0 and is not larger than `version`.
    pub const fn is_valid(self) -> bool {
        self.oldest != 0 && self.oldest <= self.version
    }
}

/// The reason why two peers cannot use a common version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionError {
    /// A range has `oldest` equal to 0 or `oldest` larger than `version`.
    InvalidRange(VersionRange),
    /// The ranges have no common version.
    Incompatible {
        /// The range of the local peer.
        local: VersionRange,
        /// The range of the remote peer.
        remote: VersionRange,
    },
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange(range) => write!(
                f,
                "version range is not valid: version {}, oldest {}",
                range.version, range.oldest
            ),
            Self::Incompatible { local, remote } => write!(
                f,
                "no common protocol version: local supports {} to {}, remote supports {} to {}",
                local.oldest, local.version, remote.oldest, remote.version
            ),
        }
    }
}

impl core::error::Error for VersionError {}

/// Selects the control protocol version for a connection.
///
/// The result is the smaller of the two `version` values. The result is an
/// error if this value is smaller than one of the two `oldest` values.
///
/// # Errors
///
/// Returns [`VersionError::InvalidRange`] if a range is not valid, and
/// [`VersionError::Incompatible`] if the ranges have no common version.
pub fn negotiate(local: VersionRange, remote: VersionRange) -> Result<u32, VersionError> {
    for range in [local, remote] {
        if !range.is_valid() {
            return Err(VersionError::InvalidRange(range));
        }
    }
    let version = local.version.min(remote.version);
    if version < local.oldest.max(remote.oldest) {
        return Err(VersionError::Incompatible { local, remote });
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn range(oldest: u32, version: u32) -> VersionRange {
        VersionRange { version, oldest }
    }

    #[test]
    fn negotiate_table() {
        // (local, remote, expected version or None for incompatible)
        let cases = [
            (range(1, 1), range(1, 1), Some(1)),
            (range(1, 3), range(1, 3), Some(3)),
            (range(1, 3), range(2, 5), Some(3)),
            (range(2, 5), range(1, 3), Some(3)),
            (range(3, 5), range(1, 3), Some(3)),
            (range(4, 5), range(1, 3), None),
            (range(1, 3), range(4, 5), None),
            (range(1, 1), range(2, 2), None),
        ];
        for (local, remote, expected) in cases {
            let result = negotiate(local, remote);
            match expected {
                Some(version) => assert_eq!(result, Ok(version), "{local:?} {remote:?}"),
                None => assert_eq!(
                    result,
                    Err(VersionError::Incompatible { local, remote }),
                    "{local:?} {remote:?}"
                ),
            }
            // The rule is symmetric.
            assert_eq!(
                negotiate(remote, local).ok(),
                expected,
                "{local:?} {remote:?}"
            );
        }
    }

    #[test]
    fn negotiate_rejects_invalid_ranges() {
        let valid = range(1, 2);
        for invalid in [range(0, 0), range(0, 2), range(3, 2)] {
            assert_eq!(
                negotiate(invalid, valid),
                Err(VersionError::InvalidRange(invalid))
            );
            assert_eq!(
                negotiate(valid, invalid),
                Err(VersionError::InvalidRange(invalid))
            );
        }
    }
}
