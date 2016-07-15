// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use colored::*;


// Raw Data Utilities ---------------------------------------------------------
macro_rules! format_chunks {
    ($data:ident, $color:ident) => {
        $data.chunks(16).map(|c| {
            c.iter().map(|d| {
                format!("0x{:0>2X}", d).$color().bold().to_string()

            }).collect::<Vec<String>>().join(", ")

        }).collect::<Vec<String>>().join(",\n        ")
    }
}

pub fn format_green(data: &[u8]) -> String {
    format_chunks!(data, green)
}

pub fn format_red(data: &[u8]) -> String {
    format_chunks!(data, red)
}

pub fn format_purple(data: &[u8]) -> String {
    format_chunks!(data, purple)
}

