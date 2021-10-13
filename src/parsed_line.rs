// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level handling of a self-contained single line key:value pair

use crate::pair::{KeyValuePair, MayContainKeyValuePair, MayContainKeyValuePairOrKeylessLine};

/// The result of parsing a single line as a key: value.
///
/// Does not handle any kind of multi-line values.
/// The multi-line version of this is [Output].
///
/// [Output]: crate::parser::Output
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLine {
    /// A line that is empty or only whitespace
    EmptyLine,
    /// A line with no key: part.
    KeylessLine(String),
    /// A proper key-value pair.
    Pair(KeyValuePair),
}

impl MayContainKeyValuePair for ParsedLine {
    fn is_pair(&self) -> bool {
        match self {
            ParsedLine::Pair(_) => true,
            ParsedLine::EmptyLine => false,
            ParsedLine::KeylessLine(_) => false,
        }
    }

    fn pair(self) -> Option<KeyValuePair> {
        match self {
            ParsedLine::Pair(pair) => Some(pair),
            ParsedLine::EmptyLine => None,
            ParsedLine::KeylessLine(_) => None,
        }
    }
}

impl MayContainKeyValuePairOrKeylessLine for ParsedLine {
    fn pair_or_err_on_keyless<E>(self, err: E) -> Result<Option<KeyValuePair>, E> {
        match self {
            ParsedLine::EmptyLine => Ok(None),
            ParsedLine::KeylessLine(_) => Err(err),
            ParsedLine::Pair(pair) => Ok(Some(pair)),
        }
    }

    fn pair_or_else_err_on_keyless<E, F: FnOnce() -> E>(
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

const DELIM: &str = ": ";

impl From<&str> for ParsedLine {
    fn from(line: &str) -> Self {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            ParsedLine::EmptyLine
        } else {
            match line.match_indices(DELIM).next() {
                Some((delim, _)) => {
                    let (k, v) = line.split_at(delim);
                    let v = &v[DELIM.len()..];

                    ParsedLine::Pair(KeyValuePair {
                        key: String::from(k),
                        value: String::from(v),
                    })
                }
                None => ParsedLine::KeylessLine(line.to_string()),
            }
        }
    }
}
