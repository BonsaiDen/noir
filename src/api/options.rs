// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::time::Duration;


/// Additional configuration options for API requests and responses.
pub struct Options {

    /// Maximum depth for recursive JSON comparison. Defaults to `4096`.
    ///
    /// For top level JSON objects their own keys and the types of their values
    /// will be at depth `0`, the actual values will then be at depth `1`.
    ///
    /// For top level JSON arrays, their length will be at depth `0` and their
    /// items and their types and values will be at level `1`.
    pub json_compare_depth: usize,

    /// Maximum duration until a HTTP request to a `Api` does time out.
    /// Defaults to `1000ms`.
    pub api_request_timeout: Duration,

    /// Whether to hide any further errors which result from expectations that
    /// are checked after the first missing or unexpected request to an
    /// external resource. Default to `true`.
    pub error_suppress_cascading: bool

}

impl Default for Options {
    fn default() -> Options {
        Options {
            json_compare_depth: 4096,
            api_request_timeout: Duration::from_millis(1000),
            error_suppress_cascading: true
        }
    }
}

