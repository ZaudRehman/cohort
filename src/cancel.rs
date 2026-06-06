//! Cancellation primitives for `cohort`.
//!
//! Cancellation is explicit and two-phase:
//! 1. graceful cancellation,
//! 2. forced abort if grace expires.

use core::fmt;
use core::time::Duration;

/// The cancellation mode currently in effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancellationMode {
    /// Graceful cooperative cancellation.
    Graceful,

    /// Forced backend-driven abort.
    Forced,
}

impl fmt::Display for CancellationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Graceful => f.write_str("Graceful"),
            Self::Forced => f.write_str("Forced"),
        }
    }
}

/// Cancellation configuration for a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cancellation {
    /// Whether graceful cancellation has been requested.
    pub requested: bool,
    /// Optional grace period before forced abort.
    pub grace: Option<Duration>,
    /// Whether forced abort has been initiated.
    pub forced: bool,
}

impl Cancellation {
    /// Creates a cancellation config with no request active.
    pub const fn new() -> Self {
        Self {
            requested: false,
            grace: None,
            forced: false,
        }
    }

    /// Creates a cancellation config with the given grace period.
    pub const fn with_grace(grace: Duration) -> Self {
        Self {
            requested: false,
            grace: Some(grace),
            forced: false,
        }
    }

    /// Requests graceful cancellation.
    pub fn request(&mut self) {
        self.requested = true;
    }

    /// Initiates forced abort.
    pub fn force(&mut self) {
        self.requested = true;
        self.forced = true;
    }

    /// Returns the current cancellation mode, if any.
    pub fn mode(&self) -> Option<CancellationMode> {
        if self.forced {
            Some(CancellationMode::Forced)
        } else if self.requested {
            Some(CancellationMode::Graceful)
        } else {
            None
        }
    }

    /// Returns `true` if cancellation has been requested.
    pub fn is_requested(&self) -> bool {
        self.requested
    }

    /// Returns `true` if forced abort has been initiated.
    pub fn is_forced(&self) -> bool {
        self.forced
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Cancellation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.requested, self.forced, self.grace) {
            (false, false, None) => f.write_str("Cancellation{idle}"),
            (true, false, Some(grace)) => {
                write!(f, "Cancellation{{graceful, grace={:?}}}", grace)
            }
            (true, false, None) => f.write_str("Cancellation{graceful}"),
            (true, true, Some(grace)) => {
                write!(f, "Cancellation{{forced, grace={:?}}}", grace)
            }
            (true, true, None) => f.write_str("Cancellation{forced}"),
            _ => f.write_str("Cancellation{idle}"),
        }
    }
}