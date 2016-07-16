// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Additional configuration options for api requests and responses.
pub struct Options {

    /// Maximum depth for recursive json comparison. Defaults to `4096`.
    ///
    /// For top level json objects their own keys and the types of their values
    /// will be at depth `0`, the actual values will then be at depth `1`.
    ///
    /// For top level json arrays, their length will be at depth `0` and their
    /// items and their types and values will be at level `1`.
    pub json_compare_depth: usize

}

impl Default for Options {
    fn default() -> Options {
        Options {
            json_compare_depth: 4096
        }
    }
}

