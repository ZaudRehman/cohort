//! Backend boundary for `cohort`.
//!
//! The backend layer provides runtime-specific execution capabilities while
//! preserving the portable structured-concurrency model in the core.

use core::time::Duration;

use crate::error::Error;
use crate::handle::Handle;
use crate::policy::Policy;

/// Backend capabilities required by the core scope model.
///
/// This trait is intentionally small and semantic. It defines the minimum set
/// of runtime operations needed by `cohort` without exposing Tokio-specific
/// types to the public surface.
pub trait Backend {
    /// Backend-specific task identifier.
    type TaskId: Copy + Eq + Send + Sync + 'static;

    /// Spawn a new task.
    fn spawn(
        &self,
        policy: Policy,
        name: Option<&str>,
        deadline: Option<Duration>,
    ) -> Result<Handle, Error>;

    /// Request cooperative cancellation.
    fn cancel(&self, task: Self::TaskId) -> Result<(), Error>;

    /// Request forced abortion.
    fn abort(&self, task: Self::TaskId) -> Result<(), Error>;

    /// Notify the backend that a task completed successfully.
    fn complete_success(&self, task: Self::TaskId) -> Result<(), Error>;

    /// Notify the backend that a task completed with an application error.
    fn complete_error(&self, task: Self::TaskId, error: &str) -> Result<(), Error>;

    /// Notify the backend that a task panicked.
    fn complete_panic(&self, task: Self::TaskId, message: &str) -> Result<(), Error>;

    /// Notify the backend that a task was cancelled.
    fn complete_cancelled(&self, task: Self::TaskId) -> Result<(), Error>;

    /// Notify the backend that a task was aborted.
    fn complete_aborted(&self, task: Self::TaskId) -> Result<(), Error>;
}

/// A backend error wrapper for runtime-specific failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    message: String,
}

impl BackendError {
    /// Creates a new backend error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for BackendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<BackendError> for Error {
    fn from(value: BackendError) -> Self {
        Error::Backend(value.message)
    }
}

#[cfg(feature = "tokio")]
pub mod tokio;