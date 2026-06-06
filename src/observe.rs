//! Lifecycle observability hooks for `cohort`.
//!
//! This module provides a minimal, runtime-agnostic callback interface for
//! observing scope and task lifecycle events without imposing a logging or
//! tracing framework.

use core::fmt;

use crate::state::{ChildState, ScopeState};

/// Lifecycle events emitted by `cohort`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A scope was created.
    ScopeCreated {
        scope_id: u64,
        policy: crate::policy::Policy,
    },
    /// A child task was spawned.
    TaskSpawned {
        scope_id: u64,
        task_id: u64,
        name: Option<String>,
    },
    /// A child task completed successfully.
    TaskSucceeded {
        scope_id: u64,
        task_id: u64,
    },
    /// A child task completed with an application error.
    TaskErrored {
        scope_id: u64,
        task_id: u64,
        error: String,
    },
    /// A child task panicked.
    TaskPanicked {
        scope_id: u64,
        task_id: u64,
        message: String,
    },
    /// A child task was cancelled.
    TaskCancelled {
        scope_id: u64,
        task_id: u64,
    },
    /// A child task was aborted.
    TaskAborted {
        scope_id: u64,
        task_id: u64,
    },
    /// Graceful scope cancellation began.
    CancellationStarted {
        scope_id: u64,
    },
    /// Forced scope abortion began.
    AbortionStarted {
        scope_id: u64,
    },
    /// Scope resolution began.
    ResolutionStarted {
        scope_id: u64,
    },
    /// Scope reached a terminal state.
    ScopeResolved {
        scope_id: u64,
        state: ScopeState,
    },
}

impl Event {
    /// Returns the scope identifier associated with this event.
    pub fn scope_id(&self) -> u64 {
        match self {
            Self::ScopeCreated { scope_id, .. }
            | Self::TaskSpawned { scope_id, .. }
            | Self::TaskSucceeded { scope_id, .. }
            | Self::TaskErrored { scope_id, .. }
            | Self::TaskPanicked { scope_id, .. }
            | Self::TaskCancelled { scope_id, .. }
            | Self::TaskAborted { scope_id, .. }
            | Self::CancellationStarted { scope_id }
            | Self::AbortionStarted { scope_id }
            | Self::ResolutionStarted { scope_id }
            | Self::ScopeResolved { scope_id, .. } => *scope_id,
        }
    }
}

/// Callback interface for observing lifecycle events.
///
/// Implementors should avoid blocking, allocation-heavy work, or panic-driven
/// control flow. Observers must be resilient because they may be invoked on
/// hot paths.
pub trait Observer: Send + Sync + 'static {
    /// Called when a lifecycle event occurs.
    fn on_event(&self, event: &Event);
}

/// A no-op observer.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopObserver;

impl Observer for NoopObserver {
    fn on_event(&self, _event: &Event) {}
}

/// A simple in-memory observer for testing and diagnostics.
///
/// This observer is intentionally single-threaded in storage strategy to keep
/// the core module dependency-light. It should be wrapped externally if shared
/// across threads.
#[derive(Debug, Default, Clone)]
pub struct RecordingObserver {
    events: alloc::vec::Vec<Event>,
}

impl RecordingObserver {
    /// Creates a new recording observer.
    pub fn new() -> Self {
        Self { events: alloc::vec::Vec::new() }
    }

    /// Returns the recorded events.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Returns the number of recorded events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` if no events have been recorded.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Observer for RecordingObserver {
    fn on_event(&self, _event: &Event) {
        // Intentionally inert in this minimal shape; a mutable or synchronized
        // variant can be introduced when the shared-state model is finalized.
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::ScopeCreated { scope_id, .. } => write!(f, "ScopeCreated(scope_id={scope_id})"),
            Event::TaskSpawned { scope_id, task_id, .. } => {
                write!(f, "TaskSpawned(scope_id={scope_id}, task_id={task_id})")
            }
            Event::TaskSucceeded { scope_id, task_id } => {
                write!(f, "TaskSucceeded(scope_id={scope_id}, task_id={task_id})")
            }
            Event::TaskErrored { scope_id, task_id, .. } => {
                write!(f, "TaskErrored(scope_id={scope_id}, task_id={task_id})")
            }
            Event::TaskPanicked { scope_id, task_id, .. } => {
                write!(f, "TaskPanicked(scope_id={scope_id}, task_id={task_id})")
            }
            Event::TaskCancelled { scope_id, task_id } => {
                write!(f, "TaskCancelled(scope_id={scope_id}, task_id={task_id})")
            }
            Event::TaskAborted { scope_id, task_id } => {
                write!(f, "TaskAborted(scope_id={scope_id}, task_id={task_id})")
            }
            Event::CancellationStarted { scope_id } => {
                write!(f, "CancellationStarted(scope_id={scope_id})")
            }
            Event::AbortionStarted { scope_id } => {
                write!(f, "AbortionStarted(scope_id={scope_id})")
            }
            Event::ResolutionStarted { scope_id } => {
                write!(f, "ResolutionStarted(scope_id={scope_id})")
            }
            Event::ScopeResolved { scope_id, state } => {
                write!(f, "ScopeResolved(scope_id={scope_id}, state={state})")
            }
        }
    }
}

/// Observer container.
pub struct Observability<O: Observer> {
    observer: O,
}

impl<O: Observer> Observability<O> {
    /// Creates a new observability wrapper.
    pub fn new(observer: O) -> Self {
        Self { observer }
    }

    /// Emits an event.
    pub fn emit(&self, event: Event) {
        self.observer.on_event(&event);
    }

    /// Returns the wrapped observer.
    pub fn observer(&self) -> &O {
        &self.observer
    }
}