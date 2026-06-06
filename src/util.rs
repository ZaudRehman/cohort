//! Internal utilities for `cohort`.
//!
//! This module contains small, deterministic helpers shared across the crate.
//! It intentionally avoids runtime coupling and hidden side effects.

use core::fmt;

/// Returns `true` if the provided identifier is valid for internal use.
///
/// A valid identifier is non-zero.
#[inline]
pub const fn is_valid_id(id: u64) -> bool {
    id != 0
}

/// Formats an optional name for display purposes.
///
/// This keeps display formatting consistent across handles, tasks, and scope
/// diagnostics.
#[inline]
pub fn format_name(name: Option<&str>) -> String {
    match name {
        Some(value) if !value.is_empty() => value.to_owned(),
        _ => "<unnamed>".to_owned(),
    }
}

/// Formats a labeled identifier in a deterministic way.
///
/// This is useful for diagnostics and debug output.
#[inline]
pub fn format_labeled_id(label: &str, id: u64) -> String {
    let mut out = String::with_capacity(label.len() + 20);
    out.push_str(label);
    out.push('(');
    out.push_str("id=");
    use core::fmt::Write as _;
    let _ = write!(&mut out, "{id}");
    out.push(')');
    out
}

/// A compact helper for rendering optional fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalDisplay<'a> {
    value: Option<&'a str>,
    empty: &'a str,
}

impl<'a> OptionalDisplay<'a> {
    /// Creates a new optional display helper.
    pub const fn new(value: Option<&'a str>, empty: &'a str) -> Self {
        Self { value, empty }
    }
}

impl fmt::Display for OptionalDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(value) if !value.is_empty() => f.write_str(value),
            _ => f.write_str(self.empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_id_non_zero() {
        assert!(is_valid_id(1));
        assert!(!is_valid_id(0));
    }

    #[test]
    fn unnamed_format() {
        assert_eq!(format_name(None), "<unnamed>");
        assert_eq!(format_name(Some("")), "<unnamed>");
        assert_eq!(format_name(Some("worker")), "worker");
    }

    #[test]
    fn labeled_id_format() {
        assert_eq!(format_labeled_id("task", 7), "task(id=7)");
    }

    #[test]
    fn optional_display_formats() {
        let a = OptionalDisplay::new(Some("alpha"), "<none>");
        let b = OptionalDisplay::new(None, "<none>");
        assert_eq!(a.to_string(), "alpha");
        assert_eq!(b.to_string(), "<none>");
    }
}