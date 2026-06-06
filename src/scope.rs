//! Scope ownership and orchestration for `cohort`.
//!
//! A scope owns child tasks, enforces policy, and provides deterministic
//! resolution semantics.

use core::fmt;
use core::time::Duration;

use crate::cancel::Cancellation;
use crate::error::Error;
use crate::handle::Handle;
use crate::policy::Policy;
use crate::state::ScopeState;
use crate::task::{Task, TaskOutcome};

/// Configuration for a scope.
///
/// This type captures the policy and cancellation settings used to govern a
/// structured task tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeConfig {
    /// Failure handling policy.
    pub policy: Policy,
    /// Cancellation configuration.
    pub cancellation: Cancellation,
    /// Optional deadline for the scope.
    pub deadline: Option<Duration>,
}

impl ScopeConfig {
    /// Creates a new scope configuration with the provided policy.
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            cancellation: Cancellation::default(),
            deadline: None,
        }
    }

    /// Sets the cancellation configuration.
    pub fn with_cancellation(mut self, cancellation: Cancellation) -> Self {
        self.cancellation = cancellation;
        self
    }

    /// Sets the deadline.
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = Some(deadline);
        self
    }
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self::new(Policy::default())
    }
}

/// Internal scope record.
///
/// This type owns task bookkeeping and deterministic lifecycle state.
#[derive(Debug)]
pub struct Scope {
    config: ScopeConfig,
    state: ScopeState,
    next_task_id: u64,
    tasks: Vec<Task>,
}

impl Scope {
    /// Creates a new scope with the given configuration.
    pub fn new(config: ScopeConfig) -> Self {
        Self {
            config,
            state: ScopeState::Open,
            next_task_id: 1,
            tasks: Vec::new(),
        }
    }

    /// Returns the current scope state.
    pub fn state(&self) -> ScopeState {
        self.state
    }

    /// Returns the policy in effect.
    pub fn policy(&self) -> Policy {
        self.config.policy
    }

    /// Returns the configured deadline, if any.
    pub fn deadline(&self) -> Option<Duration> {
        self.config.deadline
    }

    /// Returns the configured cancellation settings.
    pub fn cancellation(&self) -> Cancellation {
        self.config.cancellation
    }

    /// Returns the number of tracked children.
    pub fn child_count(&self) -> usize {
        self.tasks.len()
    }

    /// Returns `true` if the scope is still open.
    pub fn is_open(&self) -> bool {
        matches!(self.state, ScopeState::Open)
    }

    /// Returns `true` if the scope is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            ScopeState::Completed
                | ScopeState::Failed
                | ScopeState::Cancelled
                | ScopeState::TimedOut
        )
    }

    /// Spawns a new child task record and returns its handle.
    pub fn spawn(&mut self, name: Option<String>) -> Result<Handle, Error> {
        if !self.is_open() {
            return Err(Error::Cancelled);
        }

        let id = self.next_task_id;
        self.next_task_id = self
            .next_task_id
            .checked_add(1)
            .ok_or_else(|| Error::Backend("task identifier overflow".to_owned()))?;

        let task = Task::new(id, name.clone());
        let handle = Handle::new(id, name);
        self.tasks.push(task);
        Ok(handle)
    }

    /// Records a successful task completion.
    pub fn complete_success(&mut self, id: u64) -> Result<(), Error> {
        let task = self.task_mut(id)?;
        task.succeed();
        self.sync_handle(id, ChildUpdate::Success);
        Ok(())
    }

    /// Records a task error.
    pub fn complete_error(&mut self, id: u64, error: impl Into<String>) -> Result<(), Error> {
        let error = error.into();
        let task = self.task_mut(id)?;
        task.error(error.clone());
        self.sync_handle(id, ChildUpdate::Error(error));
        if matches!(self.config.policy, Policy::FailFast) {
            self.state = ScopeState::Cancelling;
        }
        Ok(())
    }

    /// Records a panic outcome.
    pub fn complete_panic(&mut self, id: u64, message: impl Into<String>) -> Result<(), Error> {
        let message = message.into();
        let task = self.task_mut(id)?;
        task.panic(message.clone());
        self.sync_handle(id, ChildUpdate::Panic(message));
        self.state = ScopeState::Cancelling;
        Ok(())
    }

    /// Records cooperative cancellation.
    pub fn complete_cancelled(&mut self, id: u64) -> Result<(), Error> {
        let task = self.task_mut(id)?;
        task.cancel();
        self.sync_handle(id, ChildUpdate::Cancelled);
        Ok(())
    }

    /// Records forced abortion.
    pub fn complete_aborted(&mut self, id: u64) -> Result<(), Error> {
        let task = self.task_mut(id)?;
        task.abort();
        self.sync_handle(id, ChildUpdate::Aborted);
        Ok(())
    }

    /// Requests graceful cancellation for the scope.
    pub fn cancel(&mut self) {
        self.config.cancellation.request();
        if self.is_open() {
            self.state = ScopeState::Cancelling;
        }
    }

    /// Requests forced abort for the scope.
    pub fn abort(&mut self) {
        self.config.cancellation.force();
        self.state = ScopeState::Aborting;
    }

    /// Marks the scope as resolving.
    pub fn begin_resolution(&mut self) {
        self.state = ScopeState::Resolving;
    }

    /// Marks the scope as completed successfully.
    pub fn mark_completed(&mut self) {
        self.state = ScopeState::Completed;
    }

    /// Marks the scope as failed.
    pub fn mark_failed(&mut self) {
        self.state = ScopeState::Failed;
    }

    /// Marks the scope as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.state = ScopeState::Cancelled;
    }

    /// Marks the scope as timed out.
    pub fn mark_timed_out(&mut self) {
        self.state = ScopeState::TimedOut;
    }

    /// Returns a read-only snapshot of a child handle.
    pub fn handle(&self, id: u64) -> Result<Handle, Error> {
        let task = self.task(id)?;
        let mut handle = Handle::new(task.id(), task.name().map(ToOwned::to_owned));
        handle.set_state(task.state());
        handle.set_outcome(task.outcome().cloned());
        Ok(handle)
    }

    fn task(&self, id: u64) -> Result<&Task, Error> {
        self.tasks
            .iter()
            .find(|task| task.id() == id)
            .ok_or_else(|| Error::Backend(format!("unknown task id {id}")))
    }

    fn task_mut(&mut self, id: u64) -> Result<&mut Task, Error> {
        self.tasks
            .iter_mut()
            .find(|task| task.id() == id)
            .ok_or_else(|| Error::Backend(format!("unknown task id {id}")))
    }

    fn sync_handle(&mut self, id: u64, update: ChildUpdate) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id() == id) {
            match update {
                ChildUpdate::Success => task.succeed(),
                ChildUpdate::Error(error) => task.error(error),
                ChildUpdate::Panic(message) => task.panic(message),
                ChildUpdate::Cancelled => task.cancel(),
                ChildUpdate::Aborted => task.abort(),
            }
        }
    }

    /// Returns a deterministic resolution summary.
    pub fn resolve(&mut self) -> Result<(), Error> {
        self.begin_resolution();

        if self.tasks.iter().any(|task| matches!(task.state(), crate::state::ChildState::Panicked)) {
            self.mark_failed();
            return Err(Error::Backend("child panic observed during resolution".to_owned()));
        }

        if self.tasks.iter().any(|task| matches!(task.state(), crate::state::ChildState::Errored)) {
            match self.config.policy {
                Policy::FailFast | Policy::Supervised => {
                    self.mark_failed();
                    return Err(Error::Backend("child error observed during resolution".to_owned()));
                }
                Policy::CollectAll => {
                    self.mark_failed();
                    return Err(Error::aggregate(Policy::CollectAll, "one or more child tasks failed"));
                }
            }
        }

        if self.tasks.iter().any(|task| matches!(task.state(), crate::state::ChildState::Cancelled)) {
            self.mark_cancelled();
            return Err(Error::Cancelled);
        }

        if self.tasks.iter().any(|task| matches!(task.state(), crate::state::ChildState::Aborted)) {
            self.mark_failed();
            return Err(Error::Backend("child task aborted".to_owned()));
        }

        self.mark_completed();
        Ok(())
    }
}

enum ChildUpdate {
    Success,
    Error(String),
    Panic(String),
    Cancelled,
    Aborted,
}

impl fmt::Display for ScopeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.deadline {
            Some(deadline) => write!(
                f,
                "ScopeConfig(policy={}, deadline={:?})",
                self.policy, deadline
            ),
            None => write!(f, "ScopeConfig(policy={})", self.policy),
        }
    }
}