// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    parse_policy::ParsePolicy, parser, record_emitter::RecordEmitter, KVParser, KeyValuePair,
    LineNumber, Output,
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

/// An ordered collection of key-value pairs, providing some helper functions above and beyond what vector provides.
pub struct Record(Vec<KeyValuePair>);

impl Default for Record {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl Record {
    /// Create from a vector.
    pub fn new(fields: Vec<KeyValuePair>) -> Self {
        Self(fields)
    }

    /// Extract the inner vector of pairs
    pub fn into_inner(self) -> Vec<KeyValuePair> {
        self.0
    }

    /// Get a shared borrow of the contained vector.
    pub fn get(&self) -> &Vec<KeyValuePair> {
        &self.0
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

impl From<Output<Vec<KeyValuePair>>> for Output<Record> {
    fn from(v: Output<Vec<KeyValuePair>>) -> Self {
        v.map(|fields| Record::new(fields))
    }
}

impl From<Vec<KeyValuePair>> for Record {
    fn from(fields: Vec<KeyValuePair>) -> Self {
        Self::new(fields)
    }
}

impl From<Record> for Vec<KeyValuePair> {
    fn from(record: Record) -> Self {
        record.into_inner()
    }
}

/// Parses key-value pairs that are grouped in "records" by an object implementing [RecordEmitter]
#[derive(Debug)]
pub struct RecordParser<R, P: ParsePolicy> {
    record_emitter: R,
    inner: KVParser<P>,
}

impl<R: RecordEmitter, P: ParsePolicy> RecordParser<R, P> {
    /// Create a new record parser, wrapping the provided parser.
    pub fn new(record_emitter: R, inner: KVParser<P>) -> Self {
        Self {
            record_emitter,
            inner,
        }
    }

    /// Pass a line to process and advance the state of the parser.
    ///
    /// If a record has finished is now available, it will
    /// be found in the return value.
    pub fn process_line(&mut self, line: &str) -> LineNumber<Output<Record>> {
        self.inner
            .process_line(line)
            .map(|v| self.record_emitter.accumulate_output(v))
            .map(|output| output.into())
    }

    /// End the input and return any record in progress
    pub fn end_input(&mut self) -> Output<Record> {
        self.record_emitter.end_input().into()
    }
}

impl<R: RecordEmitter + Default, P: ParsePolicy> RecordParser<R, P>
where
    KVParser<P>: Default,
{
    pub fn default() -> Self {
        Self {
            inner: KVParser::default(),
            record_emitter: R::default(),
        }
    }
}
