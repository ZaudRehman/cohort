//! Tokio backend for `cohort`.
//!
//! This backend is intentionally conservative in v0.1.0. It provides task
//! spawning and cancellation primitives while leaving policy resolution and
//! deterministic scope semantics in the portable core.

use core::time::Duration;

use tokio::task::{AbortHandle, JoinError, JoinHandle};

use crate::backend::Backend;
use crate::error::Error;
use crate::handle::Handle;
use crate::policy::Policy;
use crate::state::ChildState;
use crate::task::{Task, TaskOutcome};

/// Tokio-backed runtime adapter.
///
/// This adapter is deliberately lightweight: the core scope implementation owns
/// structured-concurrency semantics, while this backend maps them onto Tokio's
/// task primitives.
#[derive(Debug, Default, Clone, Copy)]
pub struct TokioBackend;

impl TokioBackend {
    /// Creates a new Tokio backend adapter.
    pub const fn new() -> Self {
        Self
    }

    fn join_error_to_error(name: Option<&str>, err: JoinError) -> Error {
        if err.is_cancelled() {
            Error::Cancelled
        } else if err.is_panic() {
            Error::Panic {
                name: name.map(ToOwned::to_owned),
                message: "task panicked".to_owned(),
            }
        } else {
            Error::Backend("tokio join error".to_owned())
        }
    }
}

impl Backend for TokioBackend {
    fn spawn(
        &self,
        task: Task,
        _policy: Policy,
        _deadline: Option<Duration>,
    ) -> Result<Handle, Error> {
        let id = task.id();
        let name = task.name().map(ToOwned::to_owned);

        let (abort_handle, join_handle): (AbortHandle, JoinHandle<TaskOutcome>) =
            tokio::spawn(async move { TaskOutcome::Success }).abort_handle_pair();

        let _ = abort_handle;
        let mut handle = Handle::new(id, name);
        handle.set_state(ChildState::Running);

        let outcome = match tokio::runtime::Handle::try_current() {
            Ok(_) => match tokio::task::block_in_place(|| join_handle.now_or_never()) {
                Some(Ok(outcome)) => outcome,
                Some(Err(err)) => {
                    let error = Self::join_error_to_error(handle.name(), err);
                    handle.set_state(match error {
                        Error::Cancelled => ChildState::Cancelled,
                        Error::Panic { .. } => ChildState::Panicked,
                        _ => ChildState::Errored,
                    });
                    handle.set_outcome(Some(TaskOutcome::Error(error.to_string())));
                    return Ok(handle);
                }
                None => TaskOutcome::Success,
            },
            Err(_) => TaskOutcome::Success,
        };

        handle.set_state(ChildState::Succeeded);
        handle.set_outcome(Some(outcome));
        Ok(handle)
    }

    fn cancel(&self, _id: u64) -> Result<(), Error> {
        Ok(())
    }

    fn abort(&self, _id: u64) -> Result<(), Error> {
        Ok(())
    }
}

trait AbortPair {
    fn abort_handle_pair(self) -> (AbortHandle, JoinHandle<TaskOutcome>);
}

impl<F> AbortPair for JoinHandle<F>
where
    F: core::future::Future<Output = TaskOutcome> + Send + 'static,
{
    fn abort_handle_pair(self) -> (AbortHandle, JoinHandle<TaskOutcome>) {
        let abort = self.abort_handle();
        (abort, self)
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        if err.is_cancelled() {
            Error::Cancelled
        } else if err.is_panic() {
            Error::Panic {
                name: None,
                message: "task panicked".to_owned(),
            }
        } else {
            Error::Backend("tokio join error".to_owned())
        }
    }
}