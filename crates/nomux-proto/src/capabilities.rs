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

/// Sessions have names: `Session.name`, the name and the reuse of
/// `Session.Create`, and `Session.Rename`. `spec/control.md`, "Names", has
/// the rules.
pub const SESSION_NAMES: &str = "session_names";

/// The daemon gives the activity of each session: `Session.last_attached_at`
/// and `Session.attached_clients`.
pub const SESSION_ACTIVITY: &str = "session_activity";

/// Sessions have a sandbox profile: `Session.sandbox`, the profile of
/// `Session.Create` and `HostInfoResponse.sandbox`. `spec/control.md`,
/// "Sandbox", has the rules.
pub const SESSION_SANDBOX: &str = "session_sandbox";
