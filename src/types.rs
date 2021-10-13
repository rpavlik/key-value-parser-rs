// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// A key-value pair.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

/// Implemented by things returned from parsing.
pub trait ParserOutput {
    type Item;

    /// Turns the contained `Item` variant into `Some(KeyValuePair)` and everything else into `None`
    ///
    /// Similar to `Option<T>::ok()`
    fn ok(self) -> Option<Self::Item>;

    /// Return an error if the output is a keyless line, otherwise extract the value if present
    ///
    /// Similar to `Option<T>::ok_or()`
    fn ok_or_err_on_keyless<E>(self, err: E) -> Result<Option<Self::Item>, E>;

    /// Call a function that returns an error if the output is a keyless line, otherwise extract the value if present.
    ///
    /// Similar to `Option<T>::ok_or_else()`
    fn ok_or_else_err_on_keyless<E, F: FnOnce() -> E>(
        self,
        err: F,
    ) -> Result<Option<Self::Item>, E>;
}

// /// Blanket implementation for anything that can be converted into a ParserOutput
// impl<T, U: ParserOutput + ?Sized> ParserOutput for T
// where
//     T: Into<U>,
// {
//     type Item = U::Item;

//     fn ok(self) -> Option<Self::Item> {
//         (self.into() as U).ok()
//     }

//     fn ok_or_err_on_keyless<E>(self, err: E) -> Result<Option<Self::Item>, E> {
//         (self.into() as U).ok_or_err_on_keyless(err)
//     }

//     fn ok_or_else_err_on_keyless<E, F: FnOnce() -> E>(
//         self,
//         err: F,
//     ) -> Result<Option<Self::Item>, E> {
//         (self.into() as U).ok_or_else_err_on_keyless(err)
//     }
// }

/// The result of parsing a single line as a key: value.
///
/// Does not handle any kind of multi-line values.
/// The multi-line version of this is [Output].
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLine {
    /// A line that is empty or only whitespace
    EmptyLine,
    /// A line with no key: part.
    KeylessLine(String),
    /// A proper key-value pair.
    Pair(KeyValuePair),
}

impl ParserOutput for ParsedLine {
    type Item = KeyValuePair;
    fn ok(self) -> Option<KeyValuePair> {
        match self {
            ParsedLine::Pair(pair) => Some(pair),
            ParsedLine::EmptyLine => None,
            ParsedLine::KeylessLine(_) => None,
        }
    }

    fn ok_or_err_on_keyless<E>(self, err: E) -> Result<Option<KeyValuePair>, E> {
        match self {
            ParsedLine::EmptyLine => Ok(None),
            ParsedLine::KeylessLine(_) => Err(err),
            ParsedLine::Pair(pair) => Ok(Some(pair)),
        }
    }

    fn ok_or_else_err_on_keyless<E, F: FnOnce() -> E>(
        self,
        err: F,
    ) -> Result<Option<KeyValuePair>, E> {
        match self {
            ParsedLine::EmptyLine => Ok(None),
            ParsedLine::KeylessLine(_) => Err(err()),
            ParsedLine::Pair(pair) => Ok(Some(pair)),
        }
    }
}

/// The output of parsing a line of input, generally by some more sophisticated parser with state.
#[derive(Debug, Clone, PartialEq)]
pub enum Output<T> {
    /// The provided line was empty or whitespace-only.
    EmptyLine,
    /// We are in the middle of some parse operation and are not ready to return a value yet
    Pending,
    /// The provided line had no key, but was not part of a multi-line value
    KeylessLine(String),
    /// The provided line completes a record
    Output(T),
}

impl<T> Output<T> {
    /// true if the value is [Output::Output]
    pub fn is_some(&self) -> bool {
        if let Output::Output(_) = self {
            true
        } else {
            false
        }
    }
    /// true if the value is [Output::Pending]
    pub fn is_pending(&self) -> bool {
        if let Output::Pending = self {
            true
        } else {
            false
        }
    }

    /// Apply a function to the contained value in the [Output::Output] variant,
    /// passing all other variants through unchanged.
    pub fn map<U, F: FnOnce(T) -> U>(self, func: F) -> Output<U> {
        match self {
            Output::EmptyLine => Output::EmptyLine,
            Output::Pending => Output::Pending,
            Output::KeylessLine(v) => Output::KeylessLine(v),
            Output::Output(v) => Output::Output(func(v)),
        }
    }
}

impl<T> Default for Output<T> {
    fn default() -> Self {
        Output::EmptyLine
    }
}

impl<T> ParserOutput for Output<T> {
    type Item = T;

    fn ok(self) -> Option<T> {
        if let Output::Output(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn ok_or_err_on_keyless<E>(self, err: E) -> Result<Option<T>, E> {
        match self {
            Output::Output(v) => Ok(Some(v)),
            Output::KeylessLine(_) => Err(err),
            _ => Ok(None),
        }
    }

    fn ok_or_else_err_on_keyless<E, F: FnOnce() -> E>(self, err: F) -> Result<Option<T>, E> {
        match self {
            Output::Output(v) => Ok(Some(v)),
            Output::KeylessLine(_) => Err(err()),
            _ => Ok(None),
        }
    }
}

impl From<ParsedLine> for Output<KeyValuePair> {
    fn from(v: ParsedLine) -> Self {
        match v {
            ParsedLine::EmptyLine => Self::EmptyLine,
            ParsedLine::KeylessLine(v) => Self::KeylessLine(v),
            ParsedLine::Pair(v) => Self::Output(v),
        }
    }
}

/// Wraps a value to add a line number field, which is typically the *last* line associated with a value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineNumber<T> {
    line_number: usize,
    value: T,
}

impl<T> LineNumber<T> {
    /// Create from a value and a line number.
    pub fn new(line_number: usize, value: T) -> Self {
        Self { line_number, value }
    }

    /// Unwrap the inner value
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get the line number
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Get the value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Convert into a (line number, value) tuple.
    pub fn into_tuple(self) -> (usize, T) {
        (self.line_number, self.value)
    }

    /// Apply a function to the contained value,
    /// returning a LineNumber with the same number but the transformed value.
    pub fn map<U, F: FnOnce(T) -> U>(self, func: F) -> LineNumber<U> {
        LineNumber {
            line_number: self.line_number,
            value: func(self.value),
        }
    }
}

impl<T: ParserOutput> ParserOutput for LineNumber<T> {
    type Item = T::Item;

    fn ok(self) -> Option<Self::Item> {
        self.value.ok()
    }

    fn ok_or_err_on_keyless<E>(self, err: E) -> Result<Option<Self::Item>, E> {
        self.value.ok_or_err_on_keyless(err)
    }

    fn ok_or_else_err_on_keyless<E, F: FnOnce() -> E>(
        self,
        err: F,
    ) -> Result<Option<Self::Item>, E> {
        self.value.ok_or_else_err_on_keyless(err)
    }
}
