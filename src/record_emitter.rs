// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Details that only affect those implementing a record emitter for [RecordParser](crate::record::RecordParser)

use crate::{KeyValuePair, Output};

/// Trait used by [RecordParser](crate::record::RecordParser) to wrap a sequence of output
/// from the [KVParser](crate::KVParser) into groups of key-value pairs serving as fields.
pub trait RecordEmitter {
    /// Handle this field parser output, updating internal state and/or emitting a record.
    fn accumulate_output(&mut self, maybe_field: Output<KeyValuePair>)
        -> Output<Vec<KeyValuePair>>;

    /// Signal the end of input, returning the record in progress if any.
    fn end_input(&mut self) -> Output<Vec<KeyValuePair>>;
}
