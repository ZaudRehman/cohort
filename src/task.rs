//! Child task representation for `cohort`.
//!
//! This module owns the internal task record used to track child lifecycle
//! and result metadata within a scope-owned task tree.

use core::fmt;

use crate::state::ChildState;

/// Internal child task record.
///
/// This type is intentionally internal: it stores task identity, name,
/// current lifecycle state, and terminal outcome metadata.
#[derive(Debug, Clone)]
pub(crate) struct Task {
    id: u64,
    name: Option<String>,
    state: ChildState,
    outcome: Option<TaskOutcome>,
}

/// Terminal outcome of a child task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskOutcome {
    /// The child completed successfully.
    Success,

    /// The child returned an application error message.
    Error(String),

    /// The child panicked with a diagnostic message.
    Panic(String),

    /// The child was cancelled cooperatively.
    Cancelled,

    /// The child was forcefully aborted.
    Aborted,
}

impl Task {
    /// Creates a new running task record.
    pub(crate) fn new(id: u64, name: Option<String>) -> Self {
        Self {
            id,
            name,
            state: ChildState::Running,
            outcome: None,
        }
    }

    /// Returns the task identifier.
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    /// Returns the task name, if available.
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the current lifecycle state.
    pub(crate) fn state(&self) -> ChildState {
        self.state
    }

    /// Returns the terminal outcome, if any.
    pub(crate) fn outcome(&self) -> Option<&TaskOutcome> {
        self.outcome.as_ref()
    }

    /// Marks the task as succeeded.
    pub(crate) fn succeed(&mut self) {
        self.state = ChildState::Succeeded;
        self.outcome = Some(TaskOutcome::Success);
    }

    /// Marks the task as errored.
    pub(crate) fn error(&mut self, error: impl Into<String>) {
        self.state = ChildState::Errored;
        self.outcome = Some(TaskOutcome::Error(error.into()));
    }

    /// Marks the task as panicked.
    pub(crate) fn panic(&mut self, message: impl Into<String>) {
        self.state = ChildState::Panicked;
        self.outcome = Some(TaskOutcome::Panic(message.into()));
    }

    /// Marks the task as cancelled.
    pub(crate) fn cancel(&mut self) {
        self.state = ChildState::Cancelled;
        self.outcome = Some(TaskOutcome::Cancelled);
    }

    /// Marks the task as aborted.
    pub(crate) fn abort(&mut self) {
        self.state = ChildState::Aborted;
        self.outcome = Some(TaskOutcome::Aborted);
    }

    /// Returns `true` if the task has reached a terminal state.
    pub(crate) fn is_terminal(&self) -> bool {
        !matches!(self.state, ChildState::Running)
    }
}

impl fmt::Display for TaskOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskOutcome::Success => f.write_str("Success"),
            TaskOutcome::Error(error) => write!(f, "Error({error})"),
            TaskOutcome::Panic(message) => write!(f, "Panic({message})"),
            TaskOutcome::Cancelled => f.write_str("Cancelled"),
            TaskOutcome::Aborted => f.write_str("Aborted"),
        }
    }
}