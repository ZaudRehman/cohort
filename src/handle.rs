//! Scope-bound child task handles.
//!
//! Handles provide read-only lifecycle inspection over a child task owned by
//! a scope. They do not allow task lifetime escape.

use core::fmt;

use crate::state::ChildState;
use crate::task::TaskOutcome;

/// A scope-bound handle to a child task.
///
/// This type is intentionally narrow. It is not a detached task handle and
/// cannot be used to outlive the scope that created it.
///
/// # Guarantees
///
/// - Scope-bound ownership.
/// - No lifetime escape.
/// - Read-only lifecycle inspection.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    id: u64,
    name: Option<String>,
    state: ChildState,
    outcome: Option<TaskOutcome>,
}

impl Handle {
    /// Creates a new handle for a named or unnamed child task.
    pub(crate) fn new(id: u64, name: Option<String>) -> Self {
        Self {
            id,
            name,
            state: ChildState::Running,
            outcome: None,
        }
    }

    /// Returns the task identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the task name, if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the current known lifecycle state.
    pub fn state(&self) -> ChildState {
        self.state
    }

    /// Returns the terminal outcome, if any.
    pub fn outcome(&self) -> Option<&TaskOutcome> {
        self.outcome.as_ref()
    }

    /// Returns `true` if the child has finished running.
    pub fn is_terminal(&self) -> bool {
        !matches!(self.state, ChildState::Running)
    }

    /// Updates the locally tracked state.
    pub(crate) fn set_state(&mut self, state: ChildState) {
        self.state = state;
    }

    /// Updates the locally tracked outcome.
    pub(crate) fn set_outcome(&mut self, outcome: Option<TaskOutcome>) {
        self.outcome = outcome;
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "Handle(id={}, name={name})", self.id),
            None => write!(f, "Handle(id={})", self.id),
        }
    }
}