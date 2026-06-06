//! Failure policy for a `cohort` scope.
//!
//! Policies define how a scope reacts when one of its children fails,
//! panics, or is cancelled.

use core::fmt;

/// Policy controlling how a scope reacts to child task failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Policy {
    /// First child failure or panic triggers cancellation of siblings.
    FailFast,

    /// Wait for all children and aggregate all failures.
    CollectAll,

    /// Record child failures without automatically cancelling siblings.
    Supervised,
}

impl Default for Policy {
    fn default() -> Self {
        Self::FailFast
    }
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailFast => f.write_str("FailFast"),
            Self::CollectAll => f.write_str("CollectAll"),
            Self::Supervised => f.write_str("Supervised"),
        }
    }
}