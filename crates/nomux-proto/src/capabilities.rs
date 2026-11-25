//! The names of the capability flags that are `active`.
//!
//! `spec/capabilities.md` is the registry of the names and has the rules. A
//! feature is active on a connection only if the `Hello` of the client and
//! the `Welcome` of the daemon have its flag.

/// The daemon can send output of an earlier epoch of a session, and the
/// frames [`HistoryGap`](crate::frame::FrameKind::HistoryGap) and
/// [`Restored`](crate::frame::FrameKind::Restored). `spec/frame.md` has the
/// rules.
pub const OUTPUT_HISTORY: &str = "output_history";
