// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    parse_policy::ParsePolicy,
    parser::{self, LineNumber},
    KVParser, KeyValuePair,
};

/// An error from operations on a Record
#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("Found {1} fields named {0} instead of the zero or one expected.")]
    WantedAtMostOneFoundMore(String, usize),

    #[error("Found {1} fields named {0} instead of the one expected.")]
    WantedOneFoundMore(String, usize),

    #[error("Missing mandatory field {0}")]
    MissingField(String),

    #[error("Out of data")]
    OutOfData,

    #[error("Other error message: {0}")]
    Message(String),
}

/// An ordered collection of key-value pairs with no (unescaped) blank lines between.
pub struct Record(Vec<KeyValuePair>);

impl Default for Record {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl Record {
    pub fn push_field(&mut self, pair: KeyValuePair) {
        self.0.push(pair)
    }

    /// Return the number of fields whose key matches the provided key
    pub fn count_fields_with_key(&self, key: &str) -> usize {
        self.0.iter().filter(|pair| pair.key == key).count()
    }

    /// Return an iterator of all field values (in original order) whose key matches the provided key
    pub fn iter_values_for_key<'a>(
        &'a self,
        key: &'a str,
    ) -> Box<dyn Iterator<Item = &'a String> + 'a> {
        Box::new(self.0.iter().filter_map(move |pair| {
            if pair.key == key {
                Some(&pair.value)
            } else {
                None
            }
        }))
    }

    /// Return a vector of all field values (in original order) whose key matches the provided key
    pub fn values_for_key<'a>(&'a self, key: &'a str) -> Vec<&'a String> {
        self.iter_values_for_key(key).collect()
    }

    /// Returns the value of a field with the given key, if any, and returns an error if more than one such field exists.
    pub fn value_for_key<'a>(&'a self, key: &'a str) -> Result<Option<&'a String>, RecordError> {
        let mut values = self.iter_values_for_key(key);
        let value = values.next();
        if values.next().is_none() {
            Ok(value)
        } else {
            Err(RecordError::WantedAtMostOneFoundMore(
                key.to_string(),
                2 + values.count(),
            ))
        }
    }
    /// Returns the value of a field with the given key, and returns an error if more than one such field exists, or if none exist.
    pub fn value_for_required_key<'a>(&'a self, key: &'a str) -> Result<&'a String, RecordError> {
        let mut values = self.iter_values_for_key(key);
        match values.next() {
            Some(value) => {
                if values.next().is_none() {
                    Ok(value)
                } else {
                    Err(RecordError::WantedOneFoundMore(
                        key.to_string(),
                        2 + values.count(),
                    ))
                }
            }
            None => Err(RecordError::MissingField(key.to_string())),
        }
    }
}

/// The output of processing a line of input in [RecordParser]
#[derive(Debug, Clone, PartialEq)]
pub enum RecordOutput {
    /// The provided line was empty or whitespace-only.
    EmptyLine,
    /// We are in the middle of a multi-line value
    ValuePending,
    /// We are in the middle of a record, between values.
    RecordPending,
    /// The provided line had no key, but was not part of a multi-line value
    KeylessLine(String),
    /// The provided line completes a record
    Record(Vec<KeyValuePair>),
}

impl From<parser::Output> for RecordOutput {
    fn from(v: parser::Output) -> Self {
        match v {
            parser::Output::EmptyLine => Self::EmptyLine,
            parser::Output::ValuePending => Self::ValuePending,
            parser::Output::KeylessLine(v) => Self::KeylessLine(v),
            parser::Output::Pair(_) => Self::RecordPending,
        }
    }
}

/// Parses key-value pairs that are grouped in blank-line-separated "records"
#[derive(Debug)]
pub struct RecordParser<P: ParsePolicy> {
    inner: KVParser<P>,
    fields: Vec<KeyValuePair>,
}

impl<P: ParsePolicy> RecordParser<P> {
    /// Create a new record parser, wrapping the provided parser.
    pub fn new(inner: KVParser<P>) -> Self {
        Self {
            inner,
            fields: vec![],
        }
    }

    /// Pass a line to process and advance the state of the parser.
    ///
    /// If a record has finished is now available, it will
    /// be found in the return value.
    pub fn process_line(&mut self, line: &str) -> LineNumber<RecordOutput> {
        let (line_number, output) = self.inner.process_line(line).into_tuple();

        let output = match output {
            parser::Output::EmptyLine => {
                if self.fields.is_empty() {
                    RecordOutput::EmptyLine
                } else {
                    RecordOutput::Record(std::mem::take(&mut self.fields))
                }
            }
            parser::Output::ValuePending => RecordOutput::ValuePending,
            parser::Output::KeylessLine(v) => RecordOutput::KeylessLine(v),
            parser::Output::Pair(v) => {
                self.fields.push(v);
                RecordOutput::RecordPending
            }
        };
        LineNumber::new(line_number, output)
    }

    /// End the input and return any record in progress
    pub fn end_input(&mut self) -> RecordOutput {
        self.process_line("").into_inner()
    }
}

impl<P: ParsePolicy> RecordParser<P>
where
    KVParser<P>: Default,
{
    pub fn default() -> Self {
        Self {
            inner: KVParser::default(),
            fields: vec![],
        }
    }
}
