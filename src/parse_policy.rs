// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Details that only affect those implementing a policy for [KVParser](crate::KVParser)

use core::fmt::Debug;

/// Enum returned by a [ParsePolicy] when processing a value.
pub enum ProcessedValue<'a> {
    /// Indicates that the provided value is complete and not continued on the following line.
    ///
    /// The data in this variant should have any multi-line decoration stripped.
    CompleteValue(&'a str),
    /// Indicates that the provided value is not complete, and that
    /// additional lines should be processed before terminating this key: value pair.
    /// If there is a value in this variant, it will be added as the first line in the overall pair value.
    ///
    /// The data in this variant should have any multi-line decoration stripped.
    StartOfMultiline(Option<&'a str>),
}

/// Enum returned by a [ParsePolicy] when processing a continuation line for a multi-line value.
pub enum ProcessedContinuationValue<'a> {
    /// Indicates that the provided value is not complete, and that
    /// additional lines should be processed before terminating this key: value pair.
    /// If there is a value in this variant, it will be added as a line to the overall pair value.
    ///
    /// The data in this variant should have any multi-line decoration stripped.
    ContinueMultiline(Option<&'a str>),
    /// Indicates that the provided value terminates the multi-line value.
    /// If there is a value in this variant, it will be added as a line to the overall pair value.
    ///
    /// The data in this variant should have any multi-line decoration stripped.
    FinishMultiline(Option<&'a str>),
}

/// Implement this policy to customize how [KVParser](crate::KVParser) works,
/// mainly regarding multi-line values.
///
/// Bundled policies are in [crate::policies]
pub trait ParsePolicy: Debug {
    /// Called when a key and value are parsed.
    ///
    /// Allows you to trim the value, as well as report
    /// that it is only the beginning of a multi-line value.
    fn process_value<'a>(&self, key: &str, value: &'a str) -> ProcessedValue<'a>;
    /// Called with each new line once [ParsePolicy::process_value] returns
    /// [ProcessedValue::StartOfMultiline].
    ///
    /// Allows you to possibly trim or drop the line, and indicate
    /// when the multi-line value has finished.
    fn process_continuation<'a>(
        &self,
        key: &str,
        continuation_line: &'a str,
    ) -> ProcessedContinuationValue<'a>;
}
