//! Structured error types for `cohort`.
//!
//! The crate-level error model is intentionally explicit. `cohort` must
//! distinguish parent errors, child errors, cancellation, timeout,
//! aggregated child failures, panic, and backend failure.

use core::fmt;

#[cfg(feature = "tokio")]
use std::collections::BTreeMap;
#[cfg(feature = "tokio")]
use std::string::String;

use thiserror::Error;

use crate::policy::Policy;

/// The structured error type returned by `cohort`.
///
/// This type preserves the distinction between application failures,
/// cancellation, timeouts, panics, and backend failures.
///
/// # Guarantees
///
/// - Error categories remain explicit.
/// - Child failure aggregation is represented structurally.
/// - Display output is human-readable.
/// - Debug output remains implementation-friendly.
///
/// # Notes
///
/// This type is intentionally broad enough to represent scope-level
/// resolution outcomes while still allowing callers to inspect the exact
/// category that occurred.
#[derive(Debug, Error)]
pub enum Error {
    /// The scope body returned an application error.
    #[error("scope body returned an application error: {0}")]
    Parent(String),

    /// A child task returned an application error.
    #[error("child task `{name}` returned an application error: {error}")]
    Child {
        /// The child task name, if available.
        name: Option<String>,
        /// The child application error message.
        error: String,
    },

    /// One or more child tasks failed under `CollectAll`.
    #[error("one or more child tasks failed under `{policy:?}`: {summary}")]
    Aggregate {
        /// The policy in effect when the aggregation occurred.
        policy: Policy,
        /// A summary of the aggregated failure set.
        summary: String,
        /// The individual failures keyed by child name when available.
        #[cfg(feature = "tokio")]
        children: BTreeMap<String, String>,
    },

    /// A child task panicked.
    #[error("child task panicked: {message}")]
    Panic {
        /// The child task name, if available.
        name: Option<String>,
        /// A panic message or diagnostic summary.
        message: String,
    },

    /// The scope was cancelled before normal completion.
    #[error("scope was cancelled")]
    Cancelled,

    /// The scope timed out.
    #[error("scope timed out")]
    TimedOut,

    /// The runtime backend failed.
    #[error("backend failure: {0}")]
    Backend(String),
}

impl Error {
    /// Returns `true` if this error represents cancellation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// Returns `true` if this error represents timeout.
    pub fn is_timed_out(&self) -> bool {
        matches!(self, Self::TimedOut)
    }

    /// Returns `true` if this error represents a backend failure.
    pub fn is_backend(&self) -> bool {
        matches!(self, Self::Backend(_))
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Backend(value.to_owned())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Backend(value)
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

#[cfg(feature = "tokio")]
impl Error {
    /// Construct an aggregated child failure error.
    pub fn aggregate(policy: Policy, summary: impl Into<String>) -> Self {
        Self::Aggregate {
            policy,
            summary: summary.into(),
            children: BTreeMap::new(),
        }
    }

    /// Attach a child failure to an aggregate error.
    pub fn push_child(&mut self, name: impl Into<String>, error: impl Into<String>) {
        if let Self::Aggregate { children, .. } = self {
            children.insert(name.into(), error.into());
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as std::error::Error>::fmt(self, f)
    }
}