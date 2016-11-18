// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::str;


// External Dependencies ------------------------------------------------------
use colored::*;
use hyper::header::{Header, Headers, HeaderView, HeaderFormat};


// Internal Dependencies ------------------------------------------------------
use util;


/// An abstraction over different `hyper::Header` implementations.
///
/// Used by the `headers![...]` macro to easily create a vector containing
/// different types that implement the `hyper::Header` trait.
///
/// # Test Failure Examples
///
/// [expanded](terminal://headers_expected_mismatch)
/// [expanded](terminal://headers_unexpected_mismatch)
pub struct HttpHeader {
    name: String,
    value: Vec<u8>
}

impl<H: Header + HeaderFormat> From<H> for HttpHeader {

    /// Converts a implementation of the `hyper::Header` trait into a abstract
    /// representation suitable for use within a `Vec`.
    fn from(header: H) -> HttpHeader {

        let mut headers = Headers::new();
        headers.set(header);

        let name = {
            headers.iter().next().unwrap().name()
        };

        HttpHeader {
            name: name.to_string(),
            value: headers.get_raw(name).unwrap()[0].clone()
        }

    }
}

pub fn http_header_into_tuple(header: HttpHeader) -> (String, Vec<u8>) {
    (header.name, header.value)
}


// Formatting -----------------------------------------------------------------
pub fn format_http_headers(headers: &Headers) -> String {

    let mut headers = headers.iter().map(|header| {
        (header.name().to_string(), header.value_string().clone())

    }).collect::<Vec<(String, String)>>();

    headers.sort();

    let max_name_length = headers.iter().map(|h| h.0.len()).max().unwrap_or(0);

    headers.into_iter().map(|(name, value)| {
        format!("{: >2$}: {}", name.cyan(), value.purple().bold(), max_name_length)

    }).collect::<Vec<String>>().join("\n        ")

}


// Validation -----------------------------------------------------------------
pub fn validate_http_headers(
    errors: &mut Vec<String>,
    context: &str,
    expected_headers: &Headers,
    unexpected_headers: &mut Vec<String>,
    actual_headers: &Headers
) {

    // Sort for stable error ordering
    let mut headers = expected_headers.iter().collect::<Vec<HeaderView>>();
    headers.sort_by(|a, b| {
        a.name().cmp(b.name())
    });

    for header in headers {
        if let Some(expected_value) = actual_headers.get_raw(header.name()) {
            let actual_value = header.value_string();
            if expected_value[0].as_slice() != actual_value.as_bytes() {

                let expected = String::from_utf8(expected_value[0].clone()).unwrap();
                let (expected, actual, diff) = util::diff::text(
                    expected.as_str(),
                    actual_value.as_str()
                );

                errors.push(format!(
                    "{} {} \"{}\" {}\n\n        \"{}\"\n\n    {}\n\n        \"{}\"\n\n    {}\n\n        \"{}\"",
                    context.yellow(),
                    "header".yellow(),
                    header.name().blue().bold(),
                    "does not match, expected:".yellow(),
                    actual.green().bold(),
                    "but got:".yellow(),
                    expected.red().bold(),
                    "difference:".yellow(),
                    diff
                ));

            }

        } else {
            errors.push(format!(
                "{} {} \"{}\" {} {}{} {}{}",
                context.yellow(),
                "header".yellow(),
                header.name().blue().bold(),
                "was expected".yellow(),
                "to be present".green().bold(),
                ", but".yellow(),
                "is missing".red().bold(),
                ".".yellow()
            ));
        }
    }

    // Sort for stable error ordering
    unexpected_headers.sort();

    for header in unexpected_headers {
        if actual_headers.get_raw(header).is_some() {
            errors.push(format!(
                "{} {} \"{}\" {} {}{} {}{}",
                context.yellow(),
                "header".yellow(),
                header.blue().bold(),
                "was expected".yellow(),
                "to be absent".green().bold(),
                ", but".yellow(),
                "is present".red().bold(),
                ".".yellow()
            ));
        }
    }

}

