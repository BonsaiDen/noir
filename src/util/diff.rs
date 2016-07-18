// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// External Dependencies ------------------------------------------------------
use colored::*;
use difference;


// Diffing Utilities ----------------------------------------------------------
pub fn text(expected: &str, actual: &str) -> (String, String, String) {

    // Escape newlines etc.
    let expected = format!("{:?}", expected);
    let expected = &expected[1..expected.len() - 1];

    let actual = format!("{:?}", actual);
    let actual = &actual[1..actual.len() - 1];

    let diff = difference::diff(expected, actual, " ").1.into_iter().map(|diff| {
        match diff {
            difference::Difference::Same(s) => s,
            difference::Difference::Rem(s) => s.white().on_red().bold().to_string(),
            difference::Difference::Add(s) => s.white().on_green().bold().to_string()
        }

    }).filter(|s| !s.is_empty()).collect::<Vec<String>>().join(" ");

    (expected.to_string(), actual.to_string(), diff)

}

