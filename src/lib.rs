#![cfg_attr(not(feature = "tokio"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::bare_urls)]

//! `cohort` provides structured concurrency for async Rust.
//!
//! The crate models concurrency as a parent-owned task tree rather than a set
//! of detached background tasks. Child work belongs to the scope that created
//! it, failure handling is policy-driven, and cancellation is explicit.
//!
//! ## Guarantees
//!
//! - Children cannot silently outlive their scope.
//! - Scope shutdown is deterministic.
//! - Cancellation is modeled explicitly.
//! - `Drop` does not block.
//! - The public API stays scope-first rather than spawn-first.
//!
//! ## Architecture
//!
//! `cohort` is designed as a portable semantic core with a Tokio-first backend.
//! Backend-specific implementation details are intentionally kept out of the
//! public surface.
//!
//! ## Example
//!
//! ```rust
//! # #[cfg(feature = "tokio")]
//! # {
//! use cohort::{scope, Policy};
//!
//! # async fn run() -> Result<(), cohort::Error> {
//! scope(Policy::FailFast, |scope| async move {
//!     let _child = scope.spawn(async { Ok::<_, cohort::Error>(()) })?;
//!     Ok::<_, cohort::Error>(())
//! })
//! .await
//! # }
//! # }
//! ```

#[cfg(feature = "tokio")]
extern crate std;

mod cancel;
mod error;
mod policy;
mod state;

#[cfg(feature = "tokio")]
mod backend;
#[cfg(feature = "tokio")]
mod handle;
#[cfg(feature = "tokio")]
mod observe;
#[cfg(feature = "tokio")]
mod scope;
#[cfg(feature = "tokio")]
mod task;
#[cfg(feature = "tokio")]
mod util;

pub use crate::error::Error;
pub use crate::policy::Policy;
pub use crate::state::{ChildState, ScopeState};

#[cfg(feature = "tokio")]
pub use crate::handle::Handle;
#[cfg(feature = "tokio")]
pub use crate::scope::{scope, Scope};