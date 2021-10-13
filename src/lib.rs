// Copyright 2021, Collabora, Ltd.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(futures)]
pub mod async_functions;

mod pair;
pub mod parse_policy;
pub mod parsed_line;
pub mod parser;
pub mod policies;
pub mod record;

pub use pair::{KeyValuePair, MayContainKeyValuePair, MayContainKeyValuePairOrKeylessLine};
pub use parsed_line::ParsedLine;
pub use parser::KVParser;
