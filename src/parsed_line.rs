// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Low-level handling of a self-contained single line key:value pair

use crate::{KeyValuePair, ParsedLine};

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
