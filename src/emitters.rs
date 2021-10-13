// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::{record_emitter::RecordEmitter, KeyValuePair, Output};

/// A record emitter that ends/emits records on a blank line.
#[derive(Debug, Default)]
pub struct BlankLineRecordEmitter {
    fields: Vec<KeyValuePair>,
}

impl BlankLineRecordEmitter {
    fn try_take(&mut self) -> Output<Vec<KeyValuePair>> {
        if self.fields.is_empty() {
            Output::EmptyLine
        } else {
            Output::Output(core::mem::take(&mut self.fields))
        }
    }
}

impl RecordEmitter for BlankLineRecordEmitter {
    fn accumulate_output(
        &mut self,
        maybe_field: Output<KeyValuePair>,
    ) -> Output<Vec<KeyValuePair>> {
        match maybe_field {
            Output::EmptyLine => self.try_take(),
            Output::Pending => Output::Pending,
            Output::KeylessLine(v) => Output::KeylessLine(v),
            Output::Output(v) => {
                self.fields.push(v);
                Output::Pending
            }
        }
    }

    fn end_input(&mut self) -> Output<Vec<KeyValuePair>> {
        self.try_take()
    }
}
