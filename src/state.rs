//! State types for `cohort`.
//!
//! These enums model the lifecycle of scopes and children in a structured
//! concurrency tree.

use core::fmt;

/// Lifecycle state for a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeState {
    /// The scope is active and accepting work.
    Open,

    /// The scope has begun graceful cancellation.
    Cancelling,

    /// The scope is aborting unfinished work forcefully.
    Aborting,

    /// The scope is resolving child completion and final outcomes.
    Resolving,

    /// The scope completed successfully.
    Completed,

    /// The scope failed with an error.
    Failed,

    /// The scope was cancelled before normal completion.
    Cancelled,

    /// The scope timed out.
    TimedOut,
}

impl fmt::Display for ScopeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScopeState::*;
        match self {
            Open => f.write_str("Open"),
            Cancelling => f.write_str("Cancelling"),
            Aborting => f.write_str("Aborting"),
            Resolving => f.write_str("Resolving"),
            Completed => f.write_str("Completed"),
            Failed => f.write_str("Failed"),
            Cancelled => f.write_str("Cancelled"),
            TimedOut => f.write_str("TimedOut"),
        }
    }
}

/// Lifecycle state for a child task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChildState {
    /// The child is actively running.
    Running,

    /// The child finished successfully.
    Succeeded,

    /// The child returned an application error.
    Errored,

    /// The child panicked.
    Panicked,

    /// The child was cancelled cooperatively.
    Cancelled,

    /// The child was forcefully aborted.
    Aborted,
}

impl fmt::Display for ChildState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChildState::*;
        match self {
            Running => f.write_str("Running"),
            Succeeded => f.write_str("Succeeded"),
            Errored => f.write_str("Errored"),
            Panicked => f.write_str("Panicked"),
            Cancelled => f.write_str("Cancelled"),
            Aborted => f.write_str("Aborted"),
        }
    }
}