// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Process lines incrementally to get key: value pairs using [KVParser]

use core::fmt::Debug;

use crate::{
    parse_policy::{ParsePolicy, ProcessedContinuationValue, ProcessedValue},
    KeyValuePair, LineNumber, Output, ParsedLine,
};

#[derive(Debug, Clone)]
enum State {
    Ready,
    AwaitingCloseText,
}

/// A parser for key-value pairs (aka tag-value files).
///
/// Parameterized on handling of values to allow different
/// policies for e.g. handling multi-line values.
#[derive(Debug)]
pub struct KVParser<P> {
    policy: P,
    state: State,
    line_num: usize,
    pending_key: String,
    value_lines: Vec<String>,
}

impl<P: ParsePolicy> KVParser<P> {
    /// Create a parser state wrapping a parse policy.
    pub fn new(policy: P) -> Self {
        Self {
            state: State::Ready,
            line_num: 0,
            pending_key: String::new(),
            value_lines: vec![],
            policy,
        }
    }
    fn maybe_push_value_line(&mut self, maybe_value: Option<&str>) {
        if let Some(value) = maybe_value {
            self.value_lines.push(value.to_string())
        }
    }
    fn take_pending(&mut self) -> KeyValuePair {
        let value = self.value_lines.join("\n");
        self.value_lines.clear();
        let key = core::mem::take(&mut self.pending_key);
        KeyValuePair { key, value }
    }

    /// The number of lines that we have processed.
    pub fn lines_processed(&self) -> usize {
        self.line_num
    }

    /// Pass a line to process and advance the state of the parser.
    ///
    /// If a complete key: value pair is now available, it will
    /// be found in the return value.
    pub fn process_line(&mut self, line: &str) -> LineNumber<Output<KeyValuePair>> {
        self.line_num += 1;

        // Match on our current state to compute our output.
        //
        // The output also uniquely determines our next state.
        let output = match &mut self.state {
            State::Ready => match ParsedLine::from(line) {
                ParsedLine::EmptyLine => Output::EmptyLine,
                ParsedLine::KeylessLine(v) => Output::KeylessLine(v),
                ParsedLine::Pair(pair) => match self.policy.process_value(&pair.key, &pair.value) {
                    ProcessedValue::CompleteValue(value) => Output::Output(KeyValuePair {
                        key: pair.key,
                        value: value.to_string(),
                    }),
                    ProcessedValue::StartOfMultiline(maybe_value) => {
                        self.pending_key = pair.key;
                        self.value_lines.clear();
                        self.maybe_push_value_line(maybe_value);
                        Output::Pending
                    }
                },
            },
            State::AwaitingCloseText => {
                match self.policy.process_continuation(&self.pending_key, line) {
                    ProcessedContinuationValue::ContinueMultiline(maybe_value) => {
                        self.maybe_push_value_line(maybe_value);
                        Output::Pending
                    }
                    ProcessedContinuationValue::FinishMultiline(maybe_value) => {
                        self.maybe_push_value_line(maybe_value);
                        Output::Output(self.take_pending())
                    }
                }
            }
        };
        self.state = if output.is_pending() {
            State::AwaitingCloseText
        } else {
            State::Ready
        };
        LineNumber::new(self.line_num, output)
    }

    /// Take the pending key: value pair, if any, and treat it as having completed.
    /// For example, this may be useful at the end of input.
    pub fn take_pending_pair(&mut self) -> Option<KeyValuePair> {
        match &self.state {
            State::Ready => None,
            State::AwaitingCloseText => {
                self.state = State::Ready;
                Some(self.take_pending())
            }
        }
    }
}

impl<P: ParsePolicy + Debug + Default> Default for KVParser<P> {
    fn default() -> Self {
        Self::new(P::default())
    }
}

#[cfg(test)]
mod test {

    use crate::parse_policy::ParsePolicy;
    use crate::policies::SPDXParsePolicy;
    use crate::policies::TrivialParsePolicy;
    use crate::ParserOutput;

    use super::KVParser;
    use super::KeyValuePair;
    use super::LineNumber;
    use super::Output;

    #[test]
    fn basics() {
        fn test_parser<P: ParsePolicy>(mut parser: KVParser<P>) {
            assert_eq!(
                parser.process_line("key1: value1"),
                LineNumber::new(
                    1,
                    Output::Output(KeyValuePair {
                        key: "key1".to_string(),
                        value: "value1".to_string(),
                    })
                )
            );
            assert_eq!(
                parser.process_line(" "),
                LineNumber::new(2, Output::EmptyLine)
            );
            assert_eq!(
                parser.process_line("key2: value2"),
                LineNumber::new(
                    3,
                    Output::Output(KeyValuePair {
                        key: "key2".to_string(),
                        value: "value2".to_string(),
                    })
                )
            );
        }
        let parser: KVParser<TrivialParsePolicy> = KVParser::default();
        test_parser(parser);

        let parser: KVParser<SPDXParsePolicy> = KVParser::default();
        test_parser(parser);
    }

    #[test]
    fn trim_same_line() {
        let mut parser: KVParser<SPDXParsePolicy> = KVParser::default();
        assert_eq!(
            parser.process_line("key: <text>value</text>").ok().unwrap(),
            KeyValuePair {
                key: "key".to_string(),
                value: "value".to_string(),
            }
        );
    }

    #[test]
    fn long_value() {
        let mut parser: KVParser<SPDXParsePolicy> = KVParser::default();
        assert!(parser.process_line("key: <text>value").ok().is_none());

        assert_eq!(parser.process_line("").into_inner(), Output::Pending);
        assert_eq!(
            parser.process_line("value</text>").ok().unwrap(),
            KeyValuePair {
                key: "key".to_string(),
                value: "value

value"
                    .to_string(),
            }
        );
        assert_eq!(parser.process_line("").into_inner(), Output::EmptyLine);
    }
}
