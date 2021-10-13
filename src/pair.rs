// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A type for key: value pairs, and traits for things that may hold them.

/// A key-value pair.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

/// Implemented by things that may hold a [KeyValuePair], but that also might not.
pub trait MayContainKeyValuePair {
    /// true if the type contains a [KeyValuePair]
    fn is_pair(&self) -> bool;

    /// Turns the contained [KeyValuePair] variant into `Some(KeyValuePair)` and everything else into `None`
    ///
    /// Similar to `Option<T>::ok()`
    fn pair(self) -> Option<KeyValuePair>;
}

/// Implemented by things that may hold a [KeyValuePair], but that also might signify a keyless line, or something else.
pub trait MayContainKeyValuePairOrKeylessLine: MayContainKeyValuePair {
    /// Return an error if the output is a keyless line, otherwise extract the pair if present
    ///
    /// Similar to `Option<T>::ok_or()`
    fn pair_or_err_on_keyless<E>(self, err: E) -> Result<Option<KeyValuePair>, E>;

    /// Call a function that returns an error if the output is a keyless line, otherwise extract the pair if present.
    ///
    /// Similar to `Option<T>::ok_or_else()`
    fn pair_or_else_err_on_keyless<E, F: FnOnce() -> E>(
        self,
        err: F,
    ) -> Result<Option<KeyValuePair>, E>;
}
